use bytes::{Buf, BufMut, BytesMut};

pub struct PacketDecoder {
    buf: BytesMut,
}

impl PacketDecoder {
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    pub fn buffer(&mut self) -> &mut BytesMut {
        &mut self.buf
    }

    pub fn try_decode(&mut self) -> Option<(i32, BytesMut)> {
        if self.buf.len() < 1 {
            return None;
        }

        let mut len_cursor = self.buf.clone();
        let packet_len = match read_var_int(&mut len_cursor) {
            Some(len) => len as usize,
            None => return None,
        };

        let header_len = self.buf.len() - len_cursor.len();
        let total_len = header_len + packet_len;

        if self.buf.len() < total_len {
            return None;
        }

        self.buf.advance(header_len);
        let mut packet_data = self.buf.split_to(packet_len);

        let id = match read_var_int(&mut packet_data) {
            Some(id) => id,
            None => return None,
        };

        Some((id, packet_data))
    }
}

pub struct PacketEncoder {
    buf: BytesMut,
}

impl PacketEncoder {
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    pub fn append_packet(&mut self, id: i32, data: &[u8]) {
        let len = var_int_len(id) + data.len();
        write_var_int(&mut self.buf, len as i32);
        write_var_int(&mut self.buf, id);
        self.buf.put_slice(data);
    }

    pub fn take(&mut self) -> BytesMut {
        self.buf.split()
    }
}

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

fn var_int_len(value: i32) -> usize {
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
