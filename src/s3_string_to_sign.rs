use chrono::{DateTime, Utc};
use reqwest::Request;
use sha2::{Digest, Sha256};

use crate::s3_constant::S3_ALGO_VALUE;
use crate::Policy;

/// Authentication Type of Request.
///
/// Ref: https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html
#[derive(Debug, Clone)]
pub enum AuthRequestType<'a> {
    /// Use Authorization Header in Request. The famous authentication type of AWS S3 Rest APIs
    ///
    /// Ref: https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-auth-using-authorization-header.html
    AuthorizationHeader((&'a Request, &'a str, DateTime<Utc>)),
    /// Use Query Params. This authentication type can support delegating the download permisison of an object
    /// for client
    ///
    /// Ref: https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
    QueryParams((&'a Request, &'a str, DateTime<Utc>)),
    /// Post Requests. This authentication type support delegating the upload permission of an object
    /// to client
    ///
    /// Ref: https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-authentication-HTTPPOST.html
    PostRequest(&'a Policy),
}

impl<'a> AuthRequestType<'a> {
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
        Self::QueryParams((req, region, date))
    }

    pub fn new_post_presigned(policy: &'a Policy) -> Self {
        Self::PostRequest(policy)
    }

    pub fn string_to_sign(self) -> String {
        match self {
            AuthRequestType::AuthorizationHeader((req, region, date)) => {
                string_to_sign(req.canonical_hex(true), region, date)
            }
            AuthRequestType::QueryParams((req, region, date)) => {
                string_to_sign(req.canonical_hex(false), region, date)
            }
            AuthRequestType::PostRequest(policy) => policy.encode(),
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
