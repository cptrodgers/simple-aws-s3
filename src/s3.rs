use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use hmac::crypto_mac::InvalidKeyLength;
use reqwest::{Method, Request, Url};

use crate::s3_constant::*;
use crate::{AuthRequestType, CanonicalRequest, Policy, Signer};

#[derive(Debug, Clone, Serialize)]
pub struct PostPresignedInfo {
    pub upload_url: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct S3 {
    bucket: String,
    region: String,
    endpoint: String,
    access_key: String,
    secret_key: String,
}

impl S3 {
    #[inline]
    pub fn new(
        bucket: impl Into<String>,
        region: impl Into<String>,
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        let bucket = bucket.into();
        let region = region.into();
        let endpoint = endpoint.into();
        let access_key = access_key.into();
        let secret_key = secret_key.into();

        Self {
            bucket,
            region,
            endpoint,
            access_key,
            secret_key,
        }
    }

    #[inline]
    pub fn private_url(&self) -> String {
        format!(
            "https://{region}.{endpoint}/{bucket}",
            region = self.region,
            bucket = self.bucket,
            endpoint = self.endpoint,
        )
    }

    #[inline]
    pub fn public_url(&self) -> String {
        format!(
            "https://{bucket}.{region}.{endpoint}",
            bucket = self.bucket,
            region = self.region,
            endpoint = self.endpoint,
        )
    }

    #[inline]
    pub fn head_object(&self, key: &str) -> Result<Request, InvalidKeyLength> {
        self.prepare_simple_object_method(key, Method::HEAD)
    }

    #[inline]
    pub fn delete_object(&self, key: &str) -> Result<Request, InvalidKeyLength> {
        self.prepare_simple_object_method(key, Method::DELETE)
    }

    #[inline]
    pub fn generate_presigned_post(
        &self,
        key: String,
        content_type: &str,
        content_length: i32,
        expire_on: Duration,
        acl: Option<&str>,
    ) -> Result<PostPresignedInfo, InvalidKeyLength> {
        let now = Utc::now();
        let formatted_row = now.format("%Y%m%dT%H%M%SZ").to_string();
        let credential = self.credential(now);

        // Prepare Params data
        let mut fields = HashMap::new();
        fields.insert("Content-Type".into(), content_type.to_string());
        fields.insert("key".into(), key);
        fields.insert(S3_ALGO_KEY.into(), S3_ALGO_VALUE.into());
        fields.insert(S3_CRED_KEY.into(), credential);
        fields.insert(S3_DATE_KEY.into(), formatted_row);
        if let Some(acl) = acl {
            fields.insert("acl".into(), acl.into());
        }

        // Calculate Policy, and Signature
        let policy = Policy::init(expire_on, &self.bucket, (0, content_length + 10), &fields);
        let string_to_sign = AuthRequestType::new_post_presigned(&policy).string_to_sign();
        let signature = self.signer().sign(now, &string_to_sign)?;

        fields.insert("policy".into(), string_to_sign);
        fields.insert(S3_SIGNATURE_KEY.into(), signature);

        Ok(PostPresignedInfo {
            upload_url: self.private_url(),
            params: fields,
        })
    }

    #[inline]
    pub fn generate_presigned_get(
        &self,
        key: &str,
        expires_on: i32,
    ) -> Result<Request, InvalidKeyLength> {
        let now = Utc::now();
        let formatted_now = now.format("%Y%m%dT%H%M%SZ").to_string();

        // Step 1: Prepare the request and query parameters
        let mut url = Url::parse(&format!(
            "{public_url}/{key}",
            public_url = self.public_url(),
            key = key,
        ))
        .unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair(S3_ALGO_KEY, S3_ALGO_VALUE)
            .append_pair(S3_CRED_KEY, &self.credential(now))
            .append_pair(S3_DATE_KEY, &formatted_now)
            .append_pair(S3_EXPIRES_KEY, &expires_on.to_string())
            .append_pair(S3_SIGNED_HEADERS_KEY, "host");

        let host = url.host().unwrap().to_string();
        let mut req = Request::new(Method::GET, url);
        req.headers_mut().insert("host", (&host).parse().unwrap());

        // Step 2: Calculate Signature and add to url query
        let string_to_sign =
            AuthRequestType::new_query_param_presigned(&req, self.region.as_str(), now)
                .string_to_sign();
        let sign = self.signer().sign(now, &string_to_sign)?;
        req.url_mut()
            .query_pairs_mut()
            .append_pair(S3_SIGNATURE_KEY, &sign);

        Ok(req)
    }

    #[inline]
    fn prepare_simple_object_method(
        &self,
        key: &str,
        method: Method,
    ) -> Result<Request, InvalidKeyLength> {
        let now = Utc::now();
        let formatted_now = now.format("%Y%m%dT%H%M%SZ").to_string();

        let url = Url::parse(&format!("{}/{}", self.public_url(), key)).unwrap();
        let host = url.host().unwrap().to_string();

        let mut req = Request::new(method, url);
        let payload = req.payload_hex();

        let headers_mut = req.headers_mut();
        headers_mut.insert("host", host.as_str().parse().unwrap());
        headers_mut.insert(S3_CONTENT_KEY, payload.as_str().parse().unwrap());
        headers_mut.insert(S3_DATE_KEY, formatted_now.as_str().parse().unwrap());

        let signed_headers = req.signed_header();
        let string_to_sign =
            AuthRequestType::new_authorization_header(&req, self.region.as_str(), now)
                .string_to_sign();
        let sign = self.signer().sign(now, &string_to_sign)?;
        let authorization = self.format_authorization(signed_headers, sign, now);
        req.headers_mut()
            .insert("Authorization", authorization.as_str().parse().unwrap());

        Ok(req)
    }

    #[inline]
    fn signer(&self) -> Signer<'_> {
        Signer::new(&self.secret_key, &self.region)
    }

    #[inline]
    fn format_authorization(
        &self,
        signed_headers: String,
        sign: String,
        date: DateTime<Utc>,
    ) -> String {
        let cred = self.credential(date);
        format!(
            "{algo} Credential={cred},SignedHeaders={signed_headers},Signature={sign}",
            algo = S3_ALGO_VALUE,
            cred = cred,
            signed_headers = signed_headers,
            sign = sign,
        )
    }

    #[inline]
    fn credential(&self, date: DateTime<Utc>) -> String {
        format!(
            "{access_key}/{date}/{region}/s3/aws4_request",
            access_key = &self.access_key,
            date = date.format("%Y%m%d").to_string(),
            region = &self.region,
        )
    }
}
