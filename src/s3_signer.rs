use chrono::{DateTime, Utc};
use hmac::crypto_mac::InvalidKeyLength;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

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
