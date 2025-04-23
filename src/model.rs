use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
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

#[derive(Debug, PartialEq)]
pub enum RequestId {
    Number(u32),
    String(String),
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RequestId::Number(n) => n.serialize(serializer),
            RequestId::String(s) => s.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;
        match value {
            Value::Number(n) => Ok(RequestId::Number(
                n.as_u64()
                    .ok_or(serde::de::Error::custom("Expect a positive integer"))?
                    as u32,
            )),
            Value::String(s) => Ok(RequestId::String(s.into())),
            _ => Err(serde::de::Error::custom(
                "Expect a positive interger or a string",
            )),
        }
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

        let _: JsonRpcVersion2_0 =
            serde_json::from_str(&serde_json::to_string(&JsonRpcVersion2_0).unwrap()).unwrap();
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

    #[test]
    fn test_requestid() {
        assert_eq!(
            serde_json::to_string(&RequestId::Number(12345)).unwrap(),
            "12345"
        );

        assert_eq!(
            serde_json::to_string(&RequestId::String("req-abc-987".to_string())).unwrap(),
            "\"req-abc-987\""
        );

        assert_eq!(serde_json::to_string(&RequestId::Number(0)).unwrap(), "0");

        assert_eq!(
            serde_json::to_string(&RequestId::String("".to_string())).unwrap(),
            "\"\""
        );

        assert_eq!(
            serde_json::from_str::<RequestId>("12345").unwrap(),
            RequestId::Number(12345)
        );

        assert_eq!(
            serde_json::from_str::<RequestId>("\"req-abc-987\"").unwrap(),
            RequestId::String("req-abc-987".to_string())
        );

        assert_eq!(
            serde_json::from_str::<RequestId>("0").unwrap(),
            RequestId::Number(0)
        );

        assert_eq!(
            serde_json::from_str::<RequestId>("\"\"").unwrap(),
            RequestId::String("".to_string())
        );

        assert_eq!(
            serde_json::from_str::<RequestId>(&format!("{}", u32::MAX)).unwrap(),
            RequestId::Number(u32::MAX)
        );

        assert_eq!(
            serde_json::from_str::<RequestId>(&format!("{}", u32::MAX as u64 + 1)).unwrap(),
            RequestId::Number(0)
        );

        let large_num_u64: u64 = 1 << 33;
        assert_eq!(
            serde_json::from_str::<RequestId>(&format!("{}", large_num_u64)).unwrap(),
            RequestId::Number(0)
        );

        assert_eq!(
            RequestId::Number(9876),
            serde_json::from_str(
                &serde_json::to_string::<RequestId>(&RequestId::Number(9876)).unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            RequestId::String("round-trip-test".to_string()),
            serde_json::from_str(
                &serde_json::to_string::<RequestId>(&RequestId::String(
                    "round-trip-test".to_string()
                ))
                .unwrap()
            )
            .unwrap()
        );

        assert!(serde_json::from_str::<RequestId>("123.45").is_err());
        assert!(
            serde_json::from_str::<RequestId>("123.45")
                .unwrap_err()
                .to_string()
                .contains("Expect a positive integer")
        );

        assert!(serde_json::from_str::<RequestId>("-10").is_err());
        assert!(
            serde_json::from_str::<RequestId>("-10")
                .unwrap_err()
                .to_string()
                .contains("Expect a positive integer")
        );

        assert!(serde_json::from_str::<RequestId>("18446744073709551616").is_err());
        assert!(
            serde_json::from_str::<RequestId>("18446744073709551616")
                .unwrap_err()
                .is_data()
        );

        assert!(serde_json::from_str::<RequestId>("true").is_err());
        assert!(
            serde_json::from_str::<RequestId>("true")
                .unwrap_err()
                .to_string()
                .contains("Expect a positive interger or a string")
        );

        assert!(serde_json::from_str::<RequestId>("null").is_err());
        assert!(
            serde_json::from_str::<RequestId>("null")
                .unwrap_err()
                .to_string()
                .contains("Expect a positive interger or a string")
        );

        assert!(serde_json::from_str::<RequestId>("[1, 2]").is_err());
        assert!(
            serde_json::from_str::<RequestId>("[1, 2]")
                .unwrap_err()
                .to_string()
                .contains("Expect a positive interger or a string")
        );

        assert!(serde_json::from_str::<RequestId>(r#"{"key": "value"}"#).is_err());
        assert!(
            serde_json::from_str::<RequestId>(r#"{"key": "value"}"#)
                .unwrap_err()
                .to_string()
                .contains("Expect a positive interger or a string")
        );
    }
}
