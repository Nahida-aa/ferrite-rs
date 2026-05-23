use bytes::{Buf, BufMut, BytesMut};

pub fn parse_packets(buf: &mut BytesMut) -> Option<(i32, BytesMut)> {
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

pub fn var_int_len(value: i32) -> usize {
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

pub fn read_var_int(buf: &mut impl bytes::Buf) -> Option<i32> {
    let mut value = 0u32;
    let mut shift = 0;
    loop {
        if buf.remaining() == 0 {
            return None;
        }
        let b = buf.get_u8();
        value |= ((b & 0x7F) as u32) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return None;
        }
    }
    Some(value as i32)
}

pub fn write_var_int(buf: &mut impl BufMut, value: i32) {
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

pub fn read_string(buf: &mut BytesMut, max_len: usize) -> Option<String> {
    let len = read_var_int(buf)? as usize;
    if len > max_len {
        return None;
    }
    if buf.len() < len {
        return None;
    }
    let s = String::from_utf8(buf[..len].to_vec()).ok()?;
    buf.advance(len);
    Some(s)
}

pub fn write_string(buf: &mut BytesMut, s: &str) {
    write_var_int(buf, s.len() as i32);
    buf.put_slice(s.as_bytes());
}

pub fn read_uuid(buf: &mut BytesMut) -> Option<uuid::Uuid> {
    if buf.len() < 16 {
        return None;
    }
    let chunk = buf.split_to(16);
    let (most_bytes, least_bytes) = chunk.split_at(8);
    let most = u64::from_be_bytes(most_bytes.try_into().ok()?);
    let least = u64::from_be_bytes(least_bytes.try_into().ok()?);
    Some(uuid::Uuid::from_u64_pair(most, least))
}

pub fn write_uuid(buf: &mut BytesMut, uuid: uuid::Uuid) {
    let (most, least) = uuid.as_u64_pair();
    buf.put_u64(most);
    buf.put_u64(least);
}
