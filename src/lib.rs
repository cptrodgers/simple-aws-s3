//! # Simple AWS S3
//! Provides the simple way to work with AWS S3.
//!
//! This package is developing while waiting the fully supported from [aws-sdk](https://github.com/awslabs/aws-sdk-rust) from Amazon.
//!
//! ### Signer (aws sig v4)
//! We support `aws-sig-v4` feature that you can take signature from your request.
//!
//! ```rust
//! use simple_aws_s3::Signer;
//! use chrono::{Utc, DateTime, TimeZone};
//!
//! const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
//! const REGION: &str = "us-east-1";
//!
//! fn main() {
//!     let date = Utc.ymd(2013, 5, 24).and_hms(0, 0, 0);
//!     let string_to_sign = r#"AWS4-HMAC-SHA256
//! 20130524T000000Z
//! 20130524/us-east-1/s3/aws4_request
//! 7344ae5b7ee6c3e7e6b0fe0640412a37625d1fbfff95c48bbb2dc43964946972"#;
//!
//!     let signer = Signer::new(SECRET_KEY, REGION);
//!     let signature = signer.sign(date, string_to_sign).unwrap();
//! }
//! ```
//!
//! ### S3 Features
//! You can generate upload information and send it for your client (for example: browser) to upload file directly to S3.
//!
//! ```rust
//! use simple_aws_s3::*;
//! use chrono::Duration;
//!
//! const ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
//! const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
//! const REGION: &str = "us-east-1";
//! const ENDPOINT: &str = "s3.amazonaws.com";
//! const BUCKET: &str = "examplebucket";
//!
//! fn main() {
//!     let s3 = S3::new(
//!         BUCKET,
//!         REGION,
//!         ENDPOINT,
//!         ACCESS_KEY,
//!         SECRET_KEY,
//!     );
//!
//!     // Generate Presigned Post for file name example.png, content type image/png, maximum 10mbs, expire link on 1 hour, and no acl
//!     let res = s3.generate_presigned_post("example.png".into(), "image/png", 10485760, Duration::seconds(3600), None).unwrap();
//!     assert_eq!(res.upload_url, "https://us-east-1.s3.amazonaws.com/examplebucket");
//!     assert!(res.params.contains_key("policy"));
//!     assert!(res.params.contains_key(S3_CRED_KEY));
//!     assert!(res.params.contains_key(S3_DATE_KEY));
//!     assert!(res.params.contains_key(S3_SIGNATURE_KEY));
//!     assert!(!res.params.contains_key("acl"));
//!
//!     // Generate Presigned Get: Link to download example.png, expire ons 1 hour
//!     let download_request = s3.generate_presigned_get("example.png", 3600).unwrap();
//!     // let res = reqwest::Client::new().execute(download_request);
//!
//!     // Get Information of Object
//!     let head_req = s3.head_object("example.png").unwrap();
//!     // let res = reqwest::Client::new().execute(head_req);
//!
//!     // Delete Object
//!     let delete_req = s3.delete_object("example.png").unwrap();
//!     // let res = reqwest::Client::new().execute(delete_req);
//!
//! }
//! ```

#[macro_use]
extern crate serde;

pub mod error;
pub mod s3;
pub mod s3_constant;
pub mod s3_post_policy;
pub mod s3_signer;
pub mod s3_string_to_sign;

// Export as main level
pub use s3::*;
pub use s3_constant::*;
pub use s3_post_policy::*;
pub use s3_signer::*;
pub use s3_string_to_sign::*;

// Export dependencies
pub mod prelude {
    pub use hmac;
    pub use reqwest;
    pub use sha2;
}
