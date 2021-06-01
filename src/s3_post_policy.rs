use std::collections::HashMap;

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct Conditions(Vec<Value>);

impl Conditions {
    pub fn new(
        content_length_range: (i32, i32),
        bucket: &str,
        fields: &HashMap<String, String>,
    ) -> Self {
        let mut conditions = Self(vec![]);

        conditions.insert_range_number(
            "content-length-range",
            content_length_range.0,
            content_length_range.1,
        );
        conditions.insert_match("bucket", bucket);
        for (key, value) in fields.iter() {
            conditions.insert_match(key, value);
        }

        conditions
    }

    pub fn insert_match(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let v = format!(r#"{{ "{}": "{}" }}"#, key.into(), value.into());
        self.0.push(serde_json::from_str(&v).unwrap());
    }

    pub fn insert_range_number(&mut self, key: impl Into<String>, from: i32, to: i32) {
        let v = format!(r#"["{}", {}, {}]"#, key.into(), from, to);
        self.0.push(serde_json::from_str(&v).unwrap());
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Policy {
    expiration: String,
    conditions: Conditions,
}

impl Policy {
    pub fn new(expiration: DateTime<Utc>, conditions: Conditions) -> Self {
        let expiration = expiration.to_rfc3339_opts(SecondsFormat::Secs, true);
        Self {
            expiration,
            conditions,
        }
    }

    pub fn init(
        expire_on: Duration,
        bucket: &str,
        content_length_range: (i32, i32),
        fields: &HashMap<String, String>,
    ) -> Self {
        let expiration = Utc::now() + expire_on;
        let conditions = Conditions::new(content_length_range, bucket, fields);
        Self::new(expiration, conditions)
    }

    pub fn encode(&self) -> String {
        base64::encode(serde_json::to_string(self).unwrap())
    }
}
