use anyhow::{Context, Result};
use bytes::BytesMut;
use core::protocol::codec::{read_var_int, write_var_int};
use core::protocol::packets::handshake::Handshake;
use core::protocol::packets::status::{
    PingRequest, PongResponse, StatusRequest, StatusResponse,
};
use serde_json::Value;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

pub struct ServerStatusReport {
    pub address: String,
    pub version_name: String,
    pub protocol_version: i32,
    pub players_online: u32,
    pub players_max: u32,
    pub description: String,
    pub latency_ms: u128,
    pub raw_json: String,
}

pub async fn query_server_status(address: &str) -> Result<ServerStatusReport> {
    let stream = TcpStream::connect(address)
        .await
        .with_context(|| format!("failed to connect to {address}"))?;
    let (mut reader, mut writer) = stream.into_split();

    let handshake = Handshake {
        protocol_version: core::protocol::packets::PROTOCOL_VERSION,
        server_address: address.to_string(),
        server_port: 25565,
        next_state: 1,
    };
    write_packet(&mut writer, Handshake::ID, &handshake.encode()).await?;
    write_packet(&mut writer, StatusRequest::ID, &StatusRequest.encode()).await?;

    let (id, mut payload) = read_packet(&mut reader).await?;
    if id != StatusResponse::ID {
        anyhow::bail!("unexpected status packet id: 0x{id:02x}");
    }
    let status = StatusResponse::decode(&mut payload).context("invalid status response")?;

    let parsed: Value =
        serde_json::from_str(&status.json).context("status response was not valid JSON")?;
    let ping_payload = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before UNIX_EPOCH")?
        .as_millis() as i64;
    let ping = PingRequest {
        payload: ping_payload,
    };
    let start = Instant::now();
    write_packet(&mut writer, PingRequest::ID, &ping.encode()).await?;

    let (pong_id, mut pong_payload) = read_packet(&mut reader).await?;
    if pong_id != PongResponse::ID {
        anyhow::bail!("unexpected pong packet id: 0x{pong_id:02x}");
    }
    let pong = PongResponse::decode(&mut pong_payload).context("invalid pong response")?;
    if pong.payload != ping.payload {
        anyhow::bail!("pong payload mismatch");
    }

    let latency_ms = start.elapsed().as_millis();

    Ok(ServerStatusReport {
        address: address.to_string(),
        version_name: parsed
            .get("version")
            .and_then(|v| v.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        protocol_version: parsed
            .get("version")
            .and_then(|v| v.get("protocol"))
            .and_then(Value::as_i64)
            .unwrap_or_default() as i32,
        players_online: parsed
            .get("players")
            .and_then(|v| v.get("online"))
            .and_then(Value::as_u64)
            .unwrap_or_default() as u32,
        players_max: parsed
            .get("players")
            .and_then(|v| v.get("max"))
            .and_then(Value::as_u64)
            .unwrap_or_default() as u32,
        description: extract_description(&parsed),
        latency_ms,
        raw_json: status.json,
    })
}

fn extract_description(value: &Value) -> String {
    match value.get("description") {
        Some(Value::String(text)) => text.clone(),
        Some(other) => other
            .get("text")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| other.to_string()),
        None => "<missing description>".to_string(),
    }
}

async fn write_packet(writer: &mut OwnedWriteHalf, id: i32, payload: &[u8]) -> Result<()> {
    let mut body = BytesMut::new();
    write_var_int(&mut body, id);
    body.extend_from_slice(payload);

    let mut frame = BytesMut::new();
    write_var_int(&mut frame, body.len() as i32);
    frame.extend_from_slice(&body);
    writer.write_all(&frame).await?;
    Ok(())
}

async fn read_packet(reader: &mut OwnedReadHalf) -> Result<(i32, BytesMut)> {
    let len = read_var_int_from_stream(reader).await? as usize;
    let mut body = BytesMut::new();
    body.resize(len, 0);
    reader.read_exact(&mut body).await?;

    let mut payload = body.clone();
    let id = read_var_int(&mut payload).context("invalid packet id")?;
    Ok((id, payload))
}

async fn read_var_int_from_stream(reader: &mut OwnedReadHalf) -> Result<i32> {
    let mut value = 0u32;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).await?;
        let b = byte[0];
        value |= ((b & 0x7F) as u32) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            anyhow::bail!("invalid varint");
        }
    }
    Ok(value as i32)
}
