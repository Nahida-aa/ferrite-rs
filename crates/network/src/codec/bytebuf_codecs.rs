use bytes::BytesMut;
use ferrite_core::protocol::codec::{read_string, write_string};

use super::stream_codec::StreamCodec;

/// Java 对照: `ByteBufCodecs.STRING_UTF8`
pub struct StringUtf8Codec {
    max_len: usize,
}

impl StreamCodec<BytesMut, String> for StringUtf8Codec {
    fn encode(&self, buf: &mut BytesMut, value: &String) {
        write_string(buf, value);
    }

    fn decode(&self, buf: &mut BytesMut) -> String {
        read_string(buf, self.max_len).expect("string_utf8: malformed string in buffer")
    }
}

/// Java 对照: `ByteBufCodecs.stringUtf8(int maxLength)`
pub fn string_utf8(max_len: usize) -> StringUtf8Codec {
    StringUtf8Codec { max_len }
}
