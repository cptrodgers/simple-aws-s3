use reqwest::StatusCode;
use simple_aws_s3::{S3};

// Before run this example, please replace s3 config below by your config.
const ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
const REGION: &str = "us-east-1";
const ENDPOINT: &str = "s3.amazonaws.com";
const BUCKET: &str = "examplebucket";

#[tokio::main]
async fn main() {
    let s3 = S3::new(BUCKET, REGION, ENDPOINT, ACCESS_KEY, SECRET_KEY);
    // Get Information of Object such as content type and content length (bytes)
    let res = s3.head_object("text.txt").await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("Content-Type").unwrap(), &"plain/text");
    assert_eq!(res.headers().get("Content-Length").unwrap(), &"11");

    // Delete Information of Object
    s3.delete_object("text.txt").await.unwrap();
}
