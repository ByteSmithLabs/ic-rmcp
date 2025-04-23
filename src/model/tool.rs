use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Cow<'static, str>>,
    pub input_schema: serde_json::Map<String, Value>,
    pub annotations: Option<ToolAnnotations>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    pub destructive_hint: Option<bool>,
    pub idempotent_hint: Option<bool>,
    pub open_world_hint: Option<bool>,
    pub read_only_hint: Option<bool>,
    pub title: Option<String>,
}

impl Tool {
    pub fn new<N, D, S>(name: N, description: D, input_schema: S) -> Self
    where
        N: Into<Cow<'static, str>>,
        D: Into<Cow<'static, str>>,
        S: Into<serde_json::Map<String, Value>>,
    {
        Tool {
            name: name.into(),
            description: Some(description.into()),
            input_schema: input_schema.into(),
            annotations: None,
        }
    }

    pub fn annotate(self, annotations: ToolAnnotations) -> Self {
        Tool {
            annotations: Some(annotations),
            ..self
        }
    }

    pub fn schema_as_json_value(&self) -> Value {
        Value::Object(self.input_schema.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_annotations_serialize() {
        assert_eq!(
            serde_json::to_string(&ToolAnnotations {
                destructive_hint: None,
                idempotent_hint: None,
                open_world_hint: None,
                read_only_hint: None,
                title: None,
            })
            .unwrap(),r#"{"destructiveHint":null,"idempotentHint":null,"openWorldHint":null,"readOnlyHint":null,"title":null}"#
        );
    }
}
