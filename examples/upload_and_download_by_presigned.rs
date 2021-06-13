use chrono::Duration;
use reqwest::multipart::{Form, Part};
use reqwest::StatusCode;
use simple_aws_s3::{PostPresignedInfo, S3};

// Before run this example, please replace s3 config below by your config.
const ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
const REGION: &str = "us-east-1";
const ENDPOINT: &str = "s3.amazonaws.com";
const BUCKET: &str = "examplebucket";

#[tokio::main]
async fn main() {
    let s3 = S3::new(BUCKET, REGION, ENDPOINT, ACCESS_KEY, SECRET_KEY);

    let key = String::from("text.txt");
    let content = "Hello world";

    // Upload by Post Presigned
    let PostPresignedInfo { upload_url, params } = s3
        .generate_presigned_post(
            key.clone(),
            "plain/text",
            10485760,
            Duration::seconds(3600),
            None,
        )
        .unwrap();
    let mut form = Form::new();
    for (key, value) in params {
        form = form.text(key, value);
    }
    let part = Part::text(content).mime_str("plain/text").unwrap();
    form = form.part("file", part);
    let res = reqwest::Client::new()
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Download by Query Param (Get Presigned)
    let download_url = s3.generate_presigned_get(&key, 3600).unwrap();
    let res = reqwest::Client::new()
        .get(&download_url)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.unwrap(), content);
}
