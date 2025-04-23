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

#[derive(Default, Serialize, Deserialize)]
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

impl ToolAnnotations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title<T>(title: T) -> Self
    where
        T: Into<String>,
    {
        ToolAnnotations {
            title: Some(title.into()),
            ..Self::default()
        }
    }

    pub fn with_read_only(self, read_only: bool) -> Self {
        ToolAnnotations {
            read_only_hint: Some(read_only),
            ..self
        }
    }

    pub fn with_destructive(self, destructive: bool) -> Self {
        ToolAnnotations {
            destructive_hint: Some(destructive),
            ..self
        }
    }

    pub fn with_idempotent(self, idempotent: bool) -> Self {
        ToolAnnotations {
            idempotent_hint: Some(idempotent),
            ..self
        }
    }

    pub fn with_open_world(self, open_world: bool) -> Self {
        ToolAnnotations {
            open_world_hint: Some(open_world),
            ..self
        }
    }
}
