use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub struct Codec<T: 'static + Send + Sync> {
    encode: Box<dyn Fn(&T) -> Value + Send + Sync>,
    decode: Box<dyn Fn(&Value) -> Result<T, String> + Send + Sync>,
}

impl<T: 'static + Send + Sync> Codec<T> {
    pub fn new(
        encode: Box<dyn Fn(&T) -> Value + Send + Sync>,
        decode: Box<dyn Fn(&Value) -> Result<T, String> + Send + Sync>,
    ) -> Self {
        Self { encode, decode }
    }

    pub fn encode(&self, value: &T) -> Value {
        (self.encode)(value)
    }

    pub fn decode(&self, value: &Value) -> Result<T, String> {
        (self.decode)(value)
    }

    pub fn field_of(self, name: &str) -> Self {
        let name = name.to_owned();
        let name2 = name.clone();
        let inner_encode = self.encode;
        let inner_decode = self.decode;
        Self::new(
            Box::new(move |value| {
                let mut obj = serde_json::Map::new();
                obj.insert(name.clone(), inner_encode(value));
                Value::Object(obj)
            }),
            Box::new(move |value| {
                let obj = value.as_object().ok_or_else(|| "expected JSON object".to_owned())?;
                let field = obj.get(&name2).ok_or_else(|| format!("missing field '{}'", name2))?;
                inner_decode(field)
            }),
        )
    }

    pub fn optional_field_of(self, name: &str, default: T) -> Self
    where
        T: Clone,
    {
        let name = name.to_owned();
        let name2 = name.clone();
        let default2 = default.clone();
        let inner_encode = self.encode;
        let inner_decode = self.decode;
        Self::new(
            Box::new(move |value| {
                let mut obj = serde_json::Map::new();
                let inner = inner_encode(value);
                if inner != inner_encode(&default2) {
                    obj.insert(name.clone(), inner);
                }
                Value::Object(obj)
            }),
            Box::new(move |value| {
                let obj = match value.as_object() {
                    Some(obj) => obj,
                    None => return Ok(default.clone()),
                };
                match obj.get(&name2) {
                    Some(field) => inner_decode(field),
                    None => Ok(default.clone()),
                }
            }),
        )
    }

    pub fn xmap<U: 'static + Send + Sync>(
        self,
        decode_fn: impl Fn(T) -> U + Clone + Send + Sync + 'static,
        encode_fn: impl Fn(&U) -> T + Clone + Send + Sync + 'static,
    ) -> Codec<U> {
        let inner_encode = self.encode;
        let inner_decode = self.decode;
        Codec::new(
            Box::new(move |value: &U| {
                let t = encode_fn(value);
                inner_encode(&t)
            }),
            Box::new(move |value| {
                inner_decode(value).map(|t| decode_fn(t))
            }),
        )
    }
}

impl<T: Serialize + DeserializeOwned + Send + Sync + 'static> Codec<T> {
    pub fn serde() -> Self {
        Self::new(
            Box::new(|v| serde_json::to_value(v).unwrap_or(Value::Null)),
            Box::new(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string())),
        )
    }
}

pub fn list_of<T: 'static + Send + Sync>(codec: Codec<T>) -> Codec<Vec<T>> {
    let inner_encode = codec.encode;
    let inner_decode = codec.decode;
    Codec::new(
        Box::new(move |values: &Vec<T>| {
            Value::Array(values.iter().map(|v| inner_encode(v)).collect())
        }),
        Box::new(move |value| {
            let arr = value.as_array().ok_or_else(|| format!("expected array, got {}", value))?;
            arr.iter().map(|v| inner_decode(v)).collect()
        }),
    )
}

pub fn pair<A: 'static + Send + Sync, B: 'static + Send + Sync>(
    codec_a: Codec<A>,
    codec_b: Codec<B>,
) -> Codec<(A, B)> {
    let encode_a = codec_a.encode;
    let decode_a = codec_a.decode;
    let encode_b = codec_b.encode;
    let decode_b = codec_b.decode;
    Codec::new(
        Box::new(move |pair: &(A, B)| {
            let mut obj = serde_json::Map::new();
            obj.insert("first".to_owned(), encode_a(&pair.0));
            obj.insert("second".to_owned(), encode_b(&pair.1));
            Value::Object(obj)
        }),
        Box::new(move |value| {
            let obj = value.as_object().ok_or_else(|| "expected object".to_owned())?;
            let a = obj.get("first")
                .ok_or_else(|| "missing first".to_owned())
                .and_then(|v| decode_a(v))?;
            let b = obj.get("second")
                .ok_or_else(|| "missing second".to_owned())
                .and_then(|v| decode_b(v))?;
            Ok((a, b))
        }),
    )
}
