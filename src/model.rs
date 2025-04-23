use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

mod tool;

#[derive(Debug)]
struct JsonRpcVersion2_0;

impl Serialize for JsonRpcVersion2_0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion2_0 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "2.0" {
            Ok(JsonRpcVersion2_0)
        } else {
            Err(serde::de::Error::custom(format!(
                "expected `{}`, got `{}`",
                "2.0", s
            )))
        }
    }
}

pub struct ProtocolVersion(Cow<'static, str>);

impl ProtocolVersion {
    pub const V_2025_03_26: Self = Self(Cow::Borrowed("2025-03-26"));
}

impl Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.as_str() == "2025-03-26" {
            return Ok(ProtocolVersion::V_2025_03_26);
        }

        Ok(ProtocolVersion(Cow::Owned(s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpcversion() {
        assert_eq!(
            serde_json::to_string(&JsonRpcVersion2_0).expect("serialization failed"),
            "\"2.0\""
        );

        let _: JsonRpcVersion2_0 = serde_json::from_str("\"2.0\"").expect("deserialization failed");

        let msg = serde_json::from_str::<JsonRpcVersion2_0>("\"1.0\"")
            .unwrap_err()
            .to_string();
        assert!(
            msg.contains("expected `2.0`, got `1.0`"),
            "unexpected error message: {}",
            msg
        );
    }

    #[test]
    fn test_protocolversion() {
        assert_eq!(
            serde_json::to_string(&ProtocolVersion::V_2025_03_26).unwrap(),
            "\"2025-03-26\""
        );

        assert!(matches!(
            (serde_json::from_str::<ProtocolVersion>("\"2025-03-26\"").unwrap()).0,
            Cow::Borrowed("2025-03-26")
        ));

        assert!(
            matches!( serde_json::from_str::<ProtocolVersion>("\"2025-01-01\"").unwrap().0, Cow::Owned(ref s) if s == "2025-01-01")
        );
    }
}
