use chrono::{DateTime, Utc};
use hmac::crypto_mac::InvalidKeyLength;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Signer Aws Sign V4
///
/// Ref: https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html#signing-request-intro
///
/// Example:
/// ```rust
/// use simple_aws_s3::Signer;
/// use chrono::{Utc, DateTime, TimeZone};
///
/// const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
/// const REGION: &str = "us-east-1";
///
/// let date = Utc.ymd(2013, 5, 24).and_hms(0, 0, 0);
/// let string_to_sign = r#"AWS4-HMAC-SHA256
/// 20130524T000000Z
/// 20130524/us-east-1/s3/aws4_request
/// 7344ae5b7ee6c3e7e6b0fe0640412a37625d1fbfff95c48bbb2dc43964946972"#;
///
/// let expected_signature = "f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41";
///
/// let signer = Signer::new(SECRET_KEY, REGION);
/// let signature = signer.sign(date, string_to_sign).unwrap();
///
/// assert_eq!(signature, expected_signature);
/// ```
#[derive(Debug, Clone)]
pub struct Signer<'s> {
    secret_key: &'s str,
    region: &'s str,
}

impl<'s> Signer<'s> {
    #[inline]
    pub fn new(secret_key: &'s str, region: &'s str) -> Self {
        Self { secret_key, region }
    }

    #[inline]
    pub fn sign(
        &self,
        date: DateTime<Utc>,
        string_to_sign: &str,
    ) -> Result<String, InvalidKeyLength> {
        let mut key = self.signing_hasher(date)?;
        key.update(string_to_sign.as_bytes());
        let msg = key.finalize().into_bytes();
        Ok(hex::encode(&msg))
    }

    #[inline]
    fn signing_hasher(&self, date: DateTime<Utc>) -> Result<HmacSha256, InvalidKeyLength> {
        // Step 1: Sign Date
        let date = date.format("%Y%m%d").to_string();
        let mut h = HmacSha256::new_from_slice(format!("AWS4{}", self.secret_key).as_bytes())?;
        h.update(date.as_bytes());
        let date_key = h.finalize().into_bytes();

        // Step 2: Sign Date and Region
        let mut date_region_h = HmacSha256::new_from_slice(&date_key)?;
        date_region_h.update(self.region.as_bytes());
        let date_region_key = date_region_h.finalize().into_bytes();

        // Step 3: Sign Date and region and Service
        let mut date_region_service_h = HmacSha256::new_from_slice(&date_region_key)?;
        date_region_service_h.update(b"s3");
        let date_region_service_key = date_region_service_h.finalize().into_bytes();

        // Step 4: Final sign
        let mut signing_key_h = HmacSha256::new_from_slice(&date_region_service_key)?;
        signing_key_h.update(b"aws4_request");
        let signing_key = signing_key_h.finalize().into_bytes();

        HmacSha256::new_from_slice(&signing_key)
    }
}
