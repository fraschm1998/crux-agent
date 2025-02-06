use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn deserialize_msgpack_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename = "_ExtStruct")]
    pub struct ExtStruct((i8, serde_bytes::ByteArray<8>));

    let ExtStruct((_, bytes)) = ExtStruct::deserialize(deserializer)?;
    let data = u64::from_be_bytes(bytes.into_array());
    let nsecs = data >> 34;
    let secs = data & ((1 << 34) - 1);

    Some(DateTime::from_timestamp(secs as i64, nsecs as u32))
        .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
}

pub fn serialize_msgpack_datetime<S>(
    datetime: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    #[serde(rename = "_ExtStruct")]
    struct ExtStruct((i8, Vec<u8>));

    if let Some(dt) = datetime {
        let secs = dt.timestamp() as u64;
        let nsecs = dt.timestamp_subsec_nanos() as u64;
        let data = (nsecs << 34) | (secs & ((1 << 34) - 1));
        let bytes = data.to_be_bytes().to_vec();

        ExtStruct((-1, bytes)).serialize(serializer)
    } else {
        serializer.serialize_none()
    }
}
