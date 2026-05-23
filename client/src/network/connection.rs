use std::io::{Read, Write};

use aes::Aes128;
use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use cfb8::cipher::KeyIvInit;

type AesCfb8Enc = cfb8::Encryptor<Aes128>;
type AesCfb8Dec = cfb8::Decryptor<Aes128>;
use ferrite_core::protocol::codec::read_var_int;
use ferrite_core::protocol::packets::config::{
    ClientBoundKnownPacks, ClientBoundPluginMessage, ClientInformation, FinishConfiguration,
    FinishConfigurationAcknowledged, RegistryData, ServerBoundKnownPacks,
};
use ferrite_core::protocol::packets::login::LoginSuccess;
use ferrite_core::protocol::packets::play::{KeepAliveC2S, KeepAliveS2C};
use flate2::read::ZlibDecoder;
use rsa::pkcs8::DecodePublicKey;
use rsa::Pkcs1v15Encrypt;
use rsa::RsaPublicKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use super::NetworkEvent;

struct Compression {
    threshold: u32,
}

pub async fn run(
    addr: &str,
    username: &str,
    events: &mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    let (mut reader, mut writer) = stream.into_split();

    let mut compression: Option<Compression> = None;
    let mut enc_cipher: Option<AesCfb8Enc> = None;
    let mut dec_cipher: Option<AesCfb8Dec> = None;

    // Handshake
    let hs = ferrite_core::protocol::packets::handshake::Handshake {
        protocol_version: ferrite_core::protocol::packets::FERRUMC_PROTOCOL,
        server_address: "127.0.0.1".to_string(),
        server_port: 25565,
        next_state: 2,
    };
    write_packet(
        &mut writer,
        ferrite_core::protocol::packets::handshake::Handshake::ID,
        &hs.encode(),
        &None,
        &mut None,
    )
    .await?;

    // Login Start
    let offline_uuid = uuid::Uuid::from_u128({
        let s = format!("OfflinePlayer:{}", username);
        s.bytes().fold(0u128, |acc, b| acc.wrapping_mul(131).wrapping_add(b as u128))
    });
    let ls = ferrite_core::protocol::packets::login::LoginStart {
        username: username.to_string(),
        uuid: offline_uuid,
    };
    write_packet(
        &mut writer,
        ferrite_core::protocol::packets::login::LoginStart::ID,
        &ls.encode(),
        &None,
        &mut None,
    )
    .await?;

    // Login phase
    let mut login_buf = BytesMut::new();
    loop {
        read_packet(&mut reader, &mut login_buf, &compression, &mut dec_cipher).await?;

        let (id, mut data) = parse_packets(&mut login_buf)
            .ok_or(anyhow::anyhow!("Failed to decode login packet"))?;

        match id {
            0x03 => {
                // Set Compression
                let threshold = read_var_int(&mut data)
                    .ok_or(anyhow::anyhow!("bad compression threshold"))? as u32;
                tracing::info!("Compression enabled, threshold={}", threshold);
                compression = Some(Compression { threshold });
            }
            0x02 => {
                // Login Success
                tracing::debug!("LoginSuccess raw data ({} bytes): {:?}",
                    data.len(), &data[..data.len().min(64)]);
                let s = LoginSuccess::decode(&mut data)
                    .ok_or(anyhow::anyhow!("failed to decode login success"))?;
                tracing::info!("Logged in as {}", s.username);
                break;
            }
            0x01 => {
                // Encryption Request
                let (_server_id, pubkey_der, verify_token) =
                    parse_encryption_request(&mut data)?;
                tracing::info!("Encryption request received");

                let shared_secret: [u8; 16] = rand::random();
                let pubkey = RsaPublicKey::from_public_key_der(&pubkey_der)
                    .map_err(|e| anyhow::anyhow!("Failed to parse RSA key: {}", e))?;

                let enc_secret = {
                    let mut rng = rand::thread_rng();
                    pubkey
                        .encrypt(&mut rng, Pkcs1v15Encrypt, &shared_secret)
                        .map_err(|e| anyhow::anyhow!("RSA encrypt failed: {}", e))?
                };
                let enc_token = {
                    let mut rng = rand::thread_rng();
                    pubkey
                        .encrypt(&mut rng, Pkcs1v15Encrypt, &verify_token)
                        .map_err(|e| anyhow::anyhow!("RSA encrypt token failed: {}", e))?
                };

                let mut resp = BytesMut::new();
                write_var_into_buf(&mut resp, enc_secret.len() as i32);
                resp.extend_from_slice(&enc_secret);
                write_var_into_buf(&mut resp, enc_token.len() as i32);
                resp.extend_from_slice(&enc_token);

                write_packet(&mut writer, 0x01, &resp, &compression, &mut None).await?;
                tracing::info!("Encryption response sent");

                enc_cipher = Some(
                    AesCfb8Enc::new(&shared_secret.into(), &shared_secret.into())
                );
                dec_cipher = Some(
                    AesCfb8Dec::new(&shared_secret.into(), &shared_secret.into())
                );
                tracing::info!("Encryption enabled");
            }
            0x00 => {
                let reason = ferrite_core::protocol::codec::read_string(&mut data, 65535)
                    .unwrap_or_else(|| format!("<decode error: {} bytes>", data.len()));
                anyhow::bail!("Login rejected: {}", reason);
            }
            other => {
                tracing::warn!("Unexpected login packet id=0x{:02x} ({} bytes)", other, data.len());
                // Don't break, continue reading
            }
        }
    }

    // In MC 1.20.5+ (protocol 766+), the Login Acknowledged packet was removed.
    // The client transitions directly to Configuration state after Login Success.
    // Login Acknowledged
    {
        let payload = ferrite_core::protocol::packets::login::LoginAcknowledged.encode();
        let id = ferrite_core::protocol::packets::login::LoginAcknowledged::ID;
        write_packet(&mut writer, id, &payload, &compression, &mut enc_cipher).await?;
    }

    events
        .send(NetworkEvent::Connected)
        .await
        .map_err(|_| anyhow::anyhow!("channel closed"))?;

    // Configuration state
    // Step 1: Send ClientInformation (required by server before it sends config data)
    {
        let client_info = ClientInformation {
            locale: "en_US".to_string(),
            view_distance: 12,
            chat_mode: 0,
            chat_colors: true,
            displayed_skin_parts: 0x7F,
            main_hand: 1,
            enable_text_filtering: false,
            allow_server_listings: true,
            particle_status: 0,
        };
        let payload = client_info.encode();
        write_packet(&mut writer, ClientInformation::ID, &payload, &compression, &mut enc_cipher).await?;
        tracing::info!("Sent ClientInformation");
    }

    // Step 2: Enter config read loop
    let mut buf = BytesMut::new();
    let mut attempts = 0u32;
    loop {
        tracing::info!("Config loop: waiting for packet...");
        match tokio::time::timeout(std::time::Duration::from_secs(3), read_packet(&mut reader, &mut buf, &compression, &mut dec_cipher)).await {
            Ok(Ok(())) => {
                tracing::info!("Config loop got data (buf len={})", buf.len());
                attempts = 0;
            }
            Ok(Err(e)) => {
                tracing::error!("Config loop error: {}", e);
                return Err(e);
            }
            Err(_timeout) => {
                tracing::warn!("Config loop timeout (#{})", attempts + 1);
                let mut peek = [0u8; 1];
                match tokio::time::timeout(std::time::Duration::from_millis(100), reader.read(&mut peek)).await {
                    Ok(Ok(0)) => { tracing::warn!("Connection closed by server"); return Ok(()); }
                    Ok(Ok(n)) => { tracing::info!("Got {} bytes after timeout: {:02x?}", n, &peek[..n]); }
                    Ok(Err(e)) => { tracing::error!("Peek error: {}", e); }
                    Err(_) => { tracing::info!("No data available (connection idle)"); }
                }
                attempts += 1;
                if attempts >= 10 {
                    tracing::warn!("Too many timeouts, disconnecting");
                    return Ok(());
                }
                continue;
            }
        }

        while let Some((id, mut data)) = parse_packets(&mut buf) {
            if id == RegistryData::ID {
                tracing::info!("RegistryData received ({} bytes)", data.len());
                let _ = RegistryData::decode(&mut data);
            } else if id == FinishConfiguration::ID {
                tracing::info!("Configuration finished, entering play state");
                let payload = FinishConfigurationAcknowledged.encode();
                write_packet(&mut writer, FinishConfigurationAcknowledged::ID, &payload, &compression, &mut enc_cipher).await?;
                run_play_loop(
                    &mut reader, &mut writer, &mut buf, &compression,
                    &mut enc_cipher, &mut dec_cipher, events,
                )
                .await?;
                return Ok(());
            } else if id == ClientBoundKnownPacks::ID {
                tracing::info!("ClientBoundKnownPacks received");
                let payload = ServerBoundKnownPacks.encode();
                write_packet(&mut writer, ServerBoundKnownPacks::ID, &payload, &compression, &mut enc_cipher).await?;
                tracing::info!("Sent ServerBoundKnownPacks");
            } else if id == ClientBoundPluginMessage::ID {
                let msg = ClientBoundPluginMessage::decode(&mut data);
                if let Some(m) = msg {
                    tracing::info!("Plugin message: channel={}, data_len={}", m.channel, m.data.len());
                }
            } else {
                tracing::warn!("Unhandled config packet id=0x{:02x} ({} bytes) data={:02x?}", id, data.len(), &data[..data.len().min(64)]);
            }
        }
    }
}

async fn run_play_loop(
    reader: &mut OwnedReadHalf,
    writer: &mut OwnedWriteHalf,
    buf: &mut BytesMut,
    compression: &Option<Compression>,
    enc_cipher: &mut Option<AesCfb8Enc>,
    dec_cipher: &mut Option<AesCfb8Dec>,
    events: &mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    // We'll handle play state packets as they arrive

    loop {
        tracing::info!("Play loop: waiting for packet...");
        match tokio::time::timeout(std::time::Duration::from_secs(3), read_packet(reader, buf, compression, dec_cipher)).await {
            Ok(Ok(())) => {
                tracing::info!("Play loop got data (buf len={})", buf.len());
            }
            Ok(Err(e)) => {
                tracing::error!("Play loop error: {}", e);
                return Err(e);
            }
            Err(_timeout) => {
                tracing::warn!("Play loop timeout - no data from server");
                let mut peek = [0u8; 1];
                match tokio::time::timeout(std::time::Duration::from_millis(100), reader.read(&mut peek)).await {
                    Ok(Ok(0)) => { tracing::warn!("Connection closed by server"); return Ok(()); }
                    Ok(Ok(n)) => { tracing::info!("Raw bytes after timeout: {:02x?}", &peek[..n]); }
                    Ok(Err(e)) => { tracing::error!("Peek error: {}", e); }
                    Err(_) => { tracing::info!("No data available (connection idle)"); }
                }
                continue;
            }
        }

        while let Some((id, mut data)) = parse_packets(buf) {
            tracing::info!("Play loop packet id=0x{:02x} ({} bytes)", id, data.len());
            match id {
                // Keep Alive
                KeepAliveS2C::ID => {
                    let ka = KeepAliveS2C::decode(&mut data)
                        .ok_or(anyhow::anyhow!("bad keepalive"))?;
                    let payload = KeepAliveC2S { id: ka.id }.encode();
                    write_packet(writer, KeepAliveC2S::ID, &payload, compression, enc_cipher).await?;
                }
                // Login Play (Join Game)
                0x2B => {
                    tracing::info!("LoginPlay received ({} bytes)", data.len());
                }
                // Player Abilities
                0x39 => {
                    tracing::info!("PlayerAbilities received ({} bytes)", data.len());
                }
                // Entity Status (OP level)
                0x1E => {
                    tracing::info!("EntityStatus received ({} bytes)", data.len());
                }
                // Synchronize Player Position
                0x41 => {
                    tracing::info!("SyncPlayerPosition received ({} bytes)", data.len());
                    if let Some(teleport_id) = ferrite_core::protocol::codec::read_var_int(&mut data) {
                        tracing::info!("Teleport ID: {}", teleport_id);
                        // Extract position and rotation
                        let x = data.get_f64();
                        let y = data.get_f64();
                        let z = data.get_f64();
                        let _vel_x = data.get_f64();
                        let _vel_y = data.get_f64();
                        let _vel_z = data.get_f64();
                        let yaw = data.get_f32();
                        let pitch = data.get_f32();
                        tracing::info!("Spawn pos: ({}, {}, {}), yaw={}, pitch={}", x, y, z, yaw, pitch);
                        // Send teleport confirm (C→S Play 0x00)
                        let mut confirm_payload = BytesMut::new();
                        ferrite_core::protocol::codec::write_var_int(&mut confirm_payload, teleport_id);
                        write_packet(writer, 0x00, &confirm_payload, compression, enc_cipher).await?;
                        tracing::info!("Sent teleport confirm");
                        // Send player position and rotation (C→S Play 0x1E)
                        let mut pos_payload = BytesMut::new();
                        pos_payload.put_f64(x);
                        pos_payload.put_f64(y);
                        pos_payload.put_f64(z);
                        pos_payload.put_f32(yaw);
                        pos_payload.put_f32(pitch);
                        pos_payload.put_u8(0u8);
                        write_packet(writer, 0x1E, &pos_payload, compression, enc_cipher).await?;
                        tracing::info!("Sent player position");
                        
                        let _ = events.send(NetworkEvent::PlayerPosition(x, y, z)).await;
                    }
                }
                _ => {
                    tracing::trace!("Unhandled config packet id=0x{:02x}", id);
                }
            }
        }
    }
}

// ── Unified packet read (handles raw / compressed / encrypted + compressed) ──

async fn read_packet(
    reader: &mut OwnedReadHalf,
    buf: &mut BytesMut,
    compression: &Option<Compression>,
    dec_cipher: &mut Option<AesCfb8Dec>,
) -> Result<()> {
    // Step 1: read raw or encrypted frame into temp buffer
    let mut tmp = BytesMut::new();
    if let Some(ref mut cipher) = dec_cipher {
        read_encrypted_frame(reader, &mut tmp, cipher).await?;
    } else {
        read_raw_frame(reader, &mut tmp).await?;
    }

    // Step 2: decompress if needed
    if let Some(ref comp) = compression {
        decompress_into(tmp, buf, comp)?;
    } else {
        buf.extend_from_slice(&tmp);
    }
    Ok(())
}

// ── Encrypted read (decrypts VarInt length + data) ──

async fn read_encrypted_frame(
    reader: &mut OwnedReadHalf,
    buf: &mut BytesMut,
    cipher: &mut AesCfb8Dec,
) -> Result<()> {
    let mut len_bytes = Vec::new();
    loop {
        let mut b = [0u8; 1];
        reader.read_exact(&mut b).await?;
        cipher.decrypt(&mut b);
        len_bytes.push(b[0]);
        if b[0] & 0x80 == 0 {
            break;
        }
    }
    let total_len = {
        let mut t = BytesMut::from(&len_bytes[..]);
        read_var_int(&mut t).ok_or(anyhow::anyhow!("bad encrypted varint"))? as usize
    };

    buf.resize(total_len, 0);
    reader.read_exact(&mut buf[..]).await?;
    cipher.decrypt(&mut buf[..]);

    // Prepend the length VarInt
    let mut out = BytesMut::from(&len_bytes[..]);
    out.extend_from_slice(&buf[..]);
    *buf = out;
    Ok(())
}

// ── Compressed / raw frame decompression ──

fn decompress_into(
    frame: BytesMut,
    buf: &mut BytesMut,
    _comp: &Compression,
) -> Result<()> {
    tracing::info!("decompress_into frame ({} bytes)", frame.len());
    // frame = VarInt(packet_length) + VarInt(data_length) + (zlib | raw)
    let mut cursor = frame.clone();
    let _packet_len = read_var_int(&mut cursor)
        .ok_or(anyhow::anyhow!("bad frame varint"))? as usize;
    let _outer_header = frame.len() - cursor.len();

    let data_length = read_var_int(&mut cursor)
        .ok_or(anyhow::anyhow!("bad data_length varint"))? as usize;
    // frame bytes consumed so far
    let consumed = frame.len() - cursor.len();

    if data_length > 0 {
        // zlib compressed
        let mut decoder = ZlibDecoder::new(&frame[consumed..]);
        let mut decompressed = vec![0u8; data_length];
        decoder.read_exact(&mut decompressed)?;
        // Write raw packet: VarInt(payload_len) + payload
        write_var_into_buf(buf, decompressed.len() as i32);
        buf.extend_from_slice(&decompressed);
    } else {
        // raw data
        let payload = &frame[consumed..];
        write_var_into_buf(buf, payload.len() as i32);
        buf.extend_from_slice(payload);
    }
    Ok(())
}

// ── Raw frame read (no compression, no encryption) ──

async fn read_raw_frame(
    reader: &mut OwnedReadHalf,
    buf: &mut BytesMut,
) -> Result<()> {
    let mut raw_varint = [0u8; 5];
    let mut vi_len = 0;
    loop {
        reader.read_exact(&mut raw_varint[vi_len..vi_len + 1]).await?;
        vi_len += 1;
        if raw_varint[vi_len - 1] & 0x80 == 0 {
            break;
        }
    }
    let packet_len = {
        let mut cursor = BytesMut::from(&raw_varint[..vi_len]);
        read_var_int(&mut cursor).ok_or(anyhow::anyhow!("bad varint"))? as usize
    };
    buf.extend_from_slice(&raw_varint[..vi_len]);
    let start = buf.len();
    buf.resize(start + packet_len, 0);
    reader.read_exact(&mut buf[start..]).await?;
    Ok(())
}

// ── Encryption Request parsing ──

fn parse_encryption_request(data: &mut BytesMut) -> Result<(String, Vec<u8>, Vec<u8>)> {
    let server_id = ferrite_core::protocol::codec::read_string(data, 32767)
        .ok_or(anyhow::anyhow!("bad server id"))?;
    let pubkey_len = read_var_int(data).ok_or(anyhow::anyhow!("bad pubkey len"))? as usize;
    let pubkey = data.split_to(pubkey_len).to_vec();
    let token_len = read_var_int(data).ok_or(anyhow::anyhow!("bad token len"))? as usize;
    let token = data.split_to(token_len).to_vec();
    Ok((server_id, pubkey, token))
}


// ── Unified packet write (raw / compressed + encrypted) ──

async fn write_packet(
    writer: &mut OwnedWriteHalf,
    id: i32,
    data: &[u8],
    compression: &Option<Compression>,
    enc_cipher: &mut Option<AesCfb8Enc>,
) -> Result<()> {
    let raw_payload = {
        let mut p = Vec::with_capacity(var_int_encoded_len(id) + data.len());
        write_var_into_buf(&mut p, id);
        p.extend_from_slice(data);
        p
    };

    // Apply compression wrapper if needed
    let mut frame = if let Some(ref comp) = compression {
        encode_compressed_frame(&raw_payload, comp)
    } else {
        let mut f = BytesMut::new();
        write_var_into_buf(&mut f, raw_payload.len() as i32);
        f.extend_from_slice(&raw_payload);
        f
    };

    // Encrypt if needed
    if let Some(ref mut cipher) = enc_cipher {
        cipher.encrypt(&mut frame);
    }

    writer.write_all(&frame).await?;
    tracing::info!("Wrote {} bytes (id=0x{:02x}, compressed={}, encrypted={})",
        frame.len(), id, compression.is_some(), enc_cipher.is_some());
    Ok(())
}

fn encode_compressed_frame(raw_payload: &[u8], comp: &Compression) -> BytesMut {
    if comp.threshold > 0 && raw_payload.len() >= comp.threshold as usize {
        // Compress
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(raw_payload).unwrap();
        let compressed = encoder.finish().unwrap();
        let mut frame = BytesMut::new();
        // Total length includes data_length varint + compressed data
        let mut data_header = Vec::new();
        write_var_into_buf(&mut data_header, raw_payload.len() as i32);
        let data_len = data_header.len() + compressed.len();
        write_var_into_buf(&mut frame, data_len as i32);
        frame.extend_from_slice(&data_header);
        frame.extend_from_slice(&compressed);
        frame
    } else {
        // Send raw, with data_length = 0
        let mut frame = BytesMut::new();
        let data_header_len = var_int_encoded_len(0);
        let total = data_header_len + raw_payload.len();
        write_var_into_buf(&mut frame, total as i32);
        write_var_into_buf(&mut frame, 0);
        frame.extend_from_slice(raw_payload);
        frame
    }
}

// ── Helpers ──

fn parse_packets(buf: &mut BytesMut) -> Option<(i32, BytesMut)> {
    let mut tmp = buf.clone();
    let packet_len = read_var_int(&mut tmp)? as usize;
    let header_len = buf.len() - tmp.len();
    let total = header_len + packet_len;
    if buf.len() < total {
        return None;
    }
    buf.advance(header_len);
    let mut packet_data = buf.split_to(packet_len);
    let id = read_var_int(&mut packet_data)?;
    Some((id, packet_data))
}

fn var_int_encoded_len(value: i32) -> usize {
    let mut val = value as u32;
    let mut len = 0;
    loop {
        len += 1;
        if val & 0xFFFFFF80 == 0 {
            return len;
        }
        val >>= 7;
    }
}

fn write_var_into_buf(buf: &mut impl bytes::BufMut, value: i32) {
    let mut val = value as u32;
    loop {
        if val & 0xFFFFFF80 == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8((val as u8) | 0x80);
        val >>= 7;
    }
}


