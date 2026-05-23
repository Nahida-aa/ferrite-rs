use std::io::{Read, Write};

use aes::Aes128;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use cfb_mode::Cfb8;
use cipher::{KeyIvInit, StreamCipher};
use ferrite_core::protocol::codec::read_var_int;
use ferrite_core::protocol::packets::config::{
    FinishConfiguration, FinishConfigurationAcknowledged, RegistryData,
};
use ferrite_core::protocol::packets::login::LoginSuccess;
use ferrite_core::protocol::packets::play::{KeepAliveC2S, KeepAliveS2C};
use flate2::read::ZlibDecoder;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::Pkcs1v15Encrypt;
use rsa::RsaPublicKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use super::NetworkEvent;

type AesCfb8 = Cfb8<Aes128>;

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
    let mut enc_cipher: Option<AesCfb8> = None;
    let mut dec_cipher: Option<AesCfb8> = None;

    // Handshake
    let hs = ferrite_core::protocol::packets::handshake::Handshake {
        protocol_version: ferrite_core::protocol::packets::FERRUMC_PROTOCOL,
        server_address: "127.0.0.1".to_string(),
        server_port: 25565,
        next_state: 2,
    };
    write_raw_packet(
        &mut writer,
        ferrite_core::protocol::packets::handshake::Handshake::ID,
        &hs.encode(),
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
    write_raw_packet(
        &mut writer,
        ferrite_core::protocol::packets::login::LoginStart::ID,
        &ls.encode(),
    )
    .await?;

    // Login phase
    let mut login_buf = BytesMut::new();
    loop {
        if let Some(ref mut cipher) = dec_cipher {
            read_encrypted_into(&mut reader, &mut login_buf, cipher).await?;
        } else if let Some(ref comp) = compression {
            read_compressed_into(&mut reader, &mut login_buf, comp).await?;
        } else {
            read_raw_packet_into(&mut reader, &mut login_buf).await?;
        }
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
            0x02 | 0x03 => {
                // Login Success
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
                let pubkey = RsaPublicKey::from_pkcs1_der(&pubkey_der)
                    .or_else(|_| {
                        RsaPublicKey::from_pkcs1_der(&strip_der_wrapper(&pubkey_der))
                    })
                    .map_err(|e| anyhow::anyhow!("Failed to parse RSA key: {}", e))?;

                let mut rng = rand::thread_rng();
                let enc_secret = pubkey
                    .encrypt(&mut rng, Pkcs1v15Encrypt, &shared_secret)
                    .map_err(|e| anyhow::anyhow!("RSA encrypt failed: {}", e))?;
                let enc_token = pubkey
                    .encrypt(&mut rng, Pkcs1v15Encrypt, &verify_token)
                    .map_err(|e| anyhow::anyhow!("RSA encrypt token failed: {}", e))?;

                let mut resp = BytesMut::new();
                write_var_into_buf(&mut resp, enc_secret.len() as i32);
                resp.extend_from_slice(&enc_secret);
                write_var_into_buf(&mut resp, enc_token.len() as i32);
                resp.extend_from_slice(&enc_token);

                write_raw_packet(&mut writer, 0x01, &resp).await?;
                tracing::info!("Encryption response sent");

                enc_cipher = Some(
                    AesCfb8::new_from_slices(&shared_secret, &shared_secret)
                        .map_err(|e| anyhow::anyhow!("AES init: {}", e))?,
                );
                dec_cipher = Some(
                    AesCfb8::new_from_slices(&shared_secret, &shared_secret)
                        .map_err(|e| anyhow::anyhow!("AES init: {}", e))?,
                );
                tracing::info!("Encryption enabled");
            }
            0x00 => {
                let reason = ferrite_core::protocol::codec::read_string(&mut data, 65535)
                    .unwrap_or_else(|| format!("<decode error: {} bytes>", data.len()));
                anyhow::bail!("Login rejected: {}", reason);
            }
            other => tracing::warn!("Unexpected login packet id={}", other),
        }
    }

    // Login Acknowledged
    {
        let payload = ferrite_core::protocol::packets::login::LoginAcknowledged.encode();
        let id = ferrite_core::protocol::packets::login::LoginAcknowledged::ID;
        if let Some(ref mut cipher) = enc_cipher {
            write_encrypted_packet(&mut writer, id, &payload, cipher).await?;
        } else {
            write_raw_packet(&mut writer, id, &payload).await?;
        }
    }

    events
        .send(NetworkEvent::Connected)
        .await
        .map_err(|_| anyhow::anyhow!("channel closed"))?;

    // Configuration / Play
    let mut buf = BytesMut::new();
    loop {
        if let Some(ref mut cipher) = dec_cipher {
            read_encrypted_into(&mut reader, &mut buf, cipher).await?;
        } else if let Some(ref comp) = compression {
            read_compressed_into(&mut reader, &mut buf, comp).await?;
        } else {
            read_raw_packet_into(&mut reader, &mut buf).await?;
        }
        while let Some((id, mut data)) = parse_packets(&mut buf) {
            match id {
                RegistryData::ID => {
                    let _ = RegistryData::decode(&mut data);
                }
                FinishConfiguration::ID => {
                    tracing::info!("Configuration finished, entering play state");
                    let payload = FinishConfigurationAcknowledged.encode();
                    let pkt_id = FinishConfigurationAcknowledged::ID;
                    if let Some(ref mut cipher) = enc_cipher {
                        write_encrypted_packet(&mut writer, pkt_id, &payload, cipher).await?;
                    } else {
                        write_raw_packet(&mut writer, pkt_id, &payload).await?;
                    }
                    run_play_loop(
                        &mut reader, &mut writer, &mut buf, &compression,
                        &mut enc_cipher, &mut dec_cipher,
                    )
                    .await?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

async fn run_play_loop(
    reader: &mut OwnedReadHalf,
    writer: &mut OwnedWriteHalf,
    buf: &mut BytesMut,
    compression: &Option<Compression>,
    enc_cipher: &mut Option<AesCfb8>,
    dec_cipher: &mut Option<AesCfb8>,
) -> Result<()> {
    loop {
        if let Some(ref mut cipher) = dec_cipher {
            read_encrypted_into(reader, buf, cipher).await?;
        } else if let Some(ref comp) = compression {
            read_compressed_into(reader, buf, comp).await?;
        } else {
            read_raw_packet_into(reader, buf).await?;
        }
        while let Some((id, mut data)) = parse_packets(buf) {
            match id {
                KeepAliveS2C::ID => {
                    let ka = KeepAliveS2C::decode(&mut data)
                        .ok_or(anyhow::anyhow!("bad keepalive"))?;
                    let payload = KeepAliveC2S { id: ka.id }.encode();
                    if let Some(ref mut cipher) = enc_cipher {
                        write_encrypted_packet(writer, KeepAliveC2S::ID, &payload, cipher).await?;
                    } else {
                        write_raw_packet(writer, KeepAliveC2S::ID, &payload).await?;
                    }
                }
                _ => {}
            }
        }
    }
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

// ── Encrypted read (decrypts individual VarInt bytes, then packet data) ──

async fn read_encrypted_into(
    reader: &mut OwnedReadHalf,
    buf: &mut BytesMut,
    cipher: &mut AesCfb8,
) -> Result<()> {
    let mut len_bytes = Vec::new();
    loop {
        let mut b = [0u8; 1];
        reader.read_exact(&mut b).await?;
        cipher.apply_keystream(&mut b);
        len_bytes.push(b[0]);
        if b[0] & 0x80 == 0 {
            break;
        }
    }
    let total_len = {
        let mut tmp = BytesMut::from(&len_bytes[..]);
        read_var_int(&mut tmp).ok_or(anyhow::anyhow!("bad encrypted varint"))? as usize
    };

    let start = buf.len();
    buf.resize(start + total_len, 0);
    reader.read_exact(&mut buf[start..]).await?;
    cipher.apply_keystream(&mut buf[start..]);

    // Prepend the decrypted VarInt length so parse_packets can work
    let mut out = BytesMut::new();
    out.extend_from_slice(&len_bytes);
    out.extend_from_slice(&buf[start..]);
    buf.truncate(start);
    buf.extend_from_slice(&out);
    Ok(())
}

// ── Compressed read ──

async fn read_compressed_into(
    reader: &mut OwnedReadHalf,
    buf: &mut BytesMut,
    comp: &Compression,
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
    let total_len = {
        let mut cursor = BytesMut::from(&raw_varint[..vi_len]);
        read_var_int(&mut cursor).ok_or(anyhow::anyhow!("bad varint"))? as usize
    };

    let mut encoded = vec![0u8; total_len];
    reader.read_exact(&mut encoded).await?;

    let mut tmp = BytesMut::from(&encoded[..]);
    let uncompressed_len = read_var_int(&mut tmp)
        .ok_or(anyhow::anyhow!("bad compression varint"))? as usize;
    let header_size = encoded.len() - tmp.len();

    if comp.threshold > 0 && uncompressed_len > 0 {
        let mut decoder = ZlibDecoder::new(&encoded[header_size..]);
        let mut decompressed = vec![0u8; uncompressed_len];
        decoder.read_exact(&mut decompressed)?;
        write_var_into_buf(buf, decompressed.len() as i32);
        buf.extend_from_slice(&decompressed);
    } else {
        let remaining = &encoded[header_size..];
        write_var_into_buf(buf, remaining.len() as i32);
        buf.extend_from_slice(remaining);
    }
    Ok(())
}

// ── Raw (uncompressed, unencrypted) packet read ──

async fn read_raw_packet_into(
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

// ── Encrypted write ──

async fn write_encrypted_packet(
    writer: &mut OwnedWriteHalf,
    id: i32,
    data: &[u8],
    cipher: &mut AesCfb8,
) -> Result<()> {
    let mut frame = encode_packet(id, data);
    cipher.apply_keystream(&mut frame);
    writer.write_all(&frame).await?;
    Ok(())
}

// ── Raw write (no encryption) ──

async fn write_raw_packet(
    writer: &mut OwnedWriteHalf,
    id: i32,
    data: &[u8],
) -> Result<()> {
    let frame = encode_packet(id, data);
    writer.write_all(&frame).await?;
    Ok(())
}

// ── Helpers ──

fn encode_packet(id: i32, data: &[u8]) -> BytesMut {
    let mut frame = BytesMut::new();
    let payload = {
        let mut tmp = Vec::with_capacity(var_int_encoded_len(id) + data.len());
        write_var_into_buf(&mut tmp, id);
        tmp.extend_from_slice(data);
        tmp
    };
    write_var_into_buf(&mut frame, payload.len() as i32);
    frame.extend_from_slice(&payload);
    frame
}

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

fn strip_der_wrapper(der: &[u8]) -> Vec<u8> {
    // Try to extract RSA public key from SubjectPublicKeyInfo DER wrapper
    if der.len() > 4 && der[0] == 0x30 {
        let mut pos = 1;
        pos += der_len_at(der, pos) as usize;
        if pos < der.len() && der[pos] == 0x30 {
            pos += 1;
            pos += der_len_at(der, pos) as usize;
        }
        if pos < der.len() && der[pos] == 0x03 {
            pos += 1;
            pos += der_len_at(der, pos) as usize;
            if pos < der.len() {
                pos += 1; // skip unused bits
                return der[pos..].to_vec();
            }
        }
    }
    der.to_vec()
}

fn der_len_at(data: &[u8], pos: usize) -> u64 {
    if pos >= data.len() {
        return 0;
    }
    if data[pos] & 0x80 == 0 {
        data[pos] as u64
    } else {
        let count = (data[pos] & 0x7F) as usize;
        let mut len = 0u64;
        for i in 0..count.min(data.len().saturating_sub(pos + 1)) {
            len = (len << 8) | data[pos + 1 + i] as u64;
        }
        len
    }
}
