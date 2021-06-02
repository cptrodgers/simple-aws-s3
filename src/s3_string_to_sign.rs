use chrono::{DateTime, Utc};
use reqwest::Request;
use sha2::{Digest, Sha256};

use crate::s3_constant::S3_ALGO_VALUE;
use crate::Policy;

#[derive(Debug, Clone)]
pub enum StringToSignType<'a> {
    AuthorizationHeader((&'a Request, &'a str, DateTime<Utc>)),
    QueryParamsPresigned((&'a Request, &'a str, DateTime<Utc>)),
    PostUploadPresigned(&'a Policy),
}

impl<'a> StringToSignType<'a> {
    pub fn new_authorization_header(
        req: &'a Request,
        region: &'a str,
        date: DateTime<Utc>,
    ) -> Self {
        Self::AuthorizationHeader((req, region, date))
    }

    pub fn new_query_param_presigned(
        req: &'a Request,
        region: &'a str,
        date: DateTime<Utc>,
    ) -> Self {
        Self::QueryParamsPresigned((req, region, date))
    }

    pub fn new_post_presigned(policy: &'a Policy) -> Self {
        Self::PostUploadPresigned(policy)
    }

    pub fn string_to_sign(self) -> String {
        match self {
            StringToSignType::AuthorizationHeader((req, region, date)) => {
                string_to_sign(req.canonical_hex(true), region, date)
            }
            StringToSignType::QueryParamsPresigned((req, region, date)) => {
                string_to_sign(req.canonical_hex(false), region, date)
            }
            StringToSignType::PostUploadPresigned(policy) => policy.encode(),
        }
    }
}

pub trait CanonicalRequest {
    fn payload_hex(&self) -> String;
    fn signed_header(&self) -> String;
    fn canonical_header(&self) -> String;
    fn canonical_hex(&self, include_payload: bool) -> String;
}

impl CanonicalRequest for Request {
    fn payload_hex(&self) -> String {
        let body = self.body().map(|b| b.as_bytes());
        let payload = if let Some(Some(payload)) = body {
            payload
        } else {
            b""
        };

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn signed_header(&self) -> String {
        self.headers()
            .iter()
            .map(|(name, _)| name.as_str().to_lowercase())
            .collect::<Vec<String>>()
            .join(";")
    }

    fn canonical_header(&self) -> String {
        let mut res = String::new();

        for (name, value) in self.headers() {
            let name = name.as_str().to_lowercase();
            let value = value.to_str().unwrap_or("").trim();
            res.push_str(&format!(
                "{headerName}:{headerValue}\n",
                headerName = name,
                headerValue = value,
            ))
        }

        res
    }

    fn canonical_hex(&self, include_payload: bool) -> String {
        let mut canonical = String::new();
        canonical.push_str(&format!("{method}\n", method = self.method().as_str()));
        canonical.push_str(&format!("{path}\n", path = self.url().path()));
        canonical.push_str(&format!(
            "{query}\n",
            query = self.url().query().unwrap_or("")
        ));
        canonical.push_str(&format!("{header}\n", header = self.canonical_header()));
        canonical.push_str(&format!(
            "{signed_headers}\n",
            signed_headers = self.signed_header()
        ));
        if include_payload {
            canonical.push_str(&self.payload_hex());
        } else {
            canonical.push_str("UNSIGNED-PAYLOAD");
        }

        let mut hasher = Sha256::new();
        hasher.update(canonical);
        let result = hasher.finalize();
        hex::encode(result)
    }
}

#[inline]
pub fn scope(region: &str, date: DateTime<Utc>) -> String {
    format!(
        "{date}/{region}/s3/aws4_request",
        date = date.format("%Y%m%d").to_string(),
        region = region,
    )
}

#[inline]
fn string_to_sign(canonical_hex: String, region: &str, date: DateTime<Utc>) -> String {
    let formatted_date = date.format("%Y%m%dT%H%M%SZ").to_string();

    let mut s = String::new();
    s.push_str(&format!("{}\n", S3_ALGO_VALUE));
    s.push_str(&format!("{}\n", formatted_date));
    s.push_str(&format!("{}\n", scope(region, date)));
    s.push_str(&canonical_hex);

    s
}
