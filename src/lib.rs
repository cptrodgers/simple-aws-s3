//! # Simple AWS S3
//! Provides the simple way to work with AWS S3. It use reqwest to execute the reqwest.
//!
//! This package is developing while waiting the fully supported from [aws-sdk](https://github.com/awslabs/aws-sdk-rust) from Amazon.
//! ### Features:
//!
//! + Post Presigned (Upload from browser)
//! + Get Presigned (Download from browser)
//! + Bucket Operations:
//! + Object Operations:
//!     + Head Object (Retrieve Information of an Object)
//!     + Delete Object
//!
//! ### Examples:
//! use chrono::Duration;
//! use reqwest::multipart::{Form, Part};
//! use reqwest::StatusCode;
//! use simple_aws_s3::{PostPresignedInfo, S3};
//!
//! // Before run this example, please replace s3 config below by your config.
//! const ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
//! const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
//! const REGION: &str = "us-east-1";
//! const ENDPOINT: &str = "s3.amazonaws.com";
//! const BUCKET: &str = "examplebucket";
//!
//! #[tokio::main]
//! async fn main() {
//!     let s3 = S3::new(BUCKET, REGION, ENDPOINT, ACCESS_KEY, SECRET_KEY);
//!
//!     let key = String::from("text.txt");
//!     let content = "Hello world";
//!
//!     // Upload by Post Presigned
//!     let PostPresignedInfo { upload_url, params } = s3
//!         .generate_presigned_post(
//!             key.clone(),
//!             "plain/text",
//!             10485760,
//!             Duration::seconds(3600),
//!             None,
//!         )
//!         .unwrap();
//!     let mut form = Form::new();
//!     for (key, value) in params {
//!         form = form.text(key, value);
//!     }
//!     let part = Part::text(content).mime_str("plain/text").unwrap();
//!     form = form.part("file", part);
//!     let res = reqwest::Client::new()
//!         .post(&upload_url)
//!         .multipart(form)
//!         .send()
//!         .await
//!         .unwrap();
//!     assert_eq!(res.status(), StatusCode::NO_CONTENT);
//!
//!     // Download by Query Param (Get Presigned)
//!     let download_url = s3.generate_presigned_get(&key, 3600).unwrap();
//!     let res = reqwest::Client::new()
//!         .get(&download_url)
//!         .send()
//!         .await
//!         .unwrap();
//!     assert_eq!(res.status(), StatusCode::OK);
//!     assert_eq!(res.text().await.unwrap(), content);
//! }
//! + [Upload/Download](https://github.com/cptrodgers/simple-aws-s3/tree/master/examples)
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
