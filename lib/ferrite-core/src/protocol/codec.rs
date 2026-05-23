use bytes::{Buf, BufMut, BytesMut};

pub type PacketId = i32;

pub fn read_var_int(buf: &mut BytesMut) -> Option<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut pos = 0;

    loop {
        if pos >= buf.len() {
            return None;
        }
        let byte = buf[pos];
        result |= ((byte & 0x7F) as i32) << shift;
        shift += 7;
        pos += 1;
        if byte & 0x80 == 0 {
            break;
        }
    }

    buf.advance(pos);
    Some(result)
}

pub fn write_var_int(buf: &mut BytesMut, value: i32) {
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
    if len > max_len || len > buf.len() {
        return None;
    }
    let s = String::from_utf8(buf[..len].to_vec()).ok()?;
    buf.advance(len);
    Some(s)
}
