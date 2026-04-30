//! 请求转发器

use anyhow::Result;
use reqwest::Client;
use hyper::Request;
use http_body_util::BodyExt;

/// 请求转发器
pub struct Forwarder {
    client: Client,
}

impl Forwarder {
    /// 创建新的转发器
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 转发请求到目标 URL
    pub async fn forward(
        &self,
        target_url: &str,
        request: Request<hyper::body::Incoming>,
    ) -> Result<reqwest::Response> {
        // 构建新的请求
        let mut req_builder = self.client
            .request(
                reqwest::Method::from_bytes(request.method().as_str().as_bytes()).unwrap(),
                target_url,
            );

        // 复制 headers
        for (name, value) in request.headers() {
            req_builder = req_builder.header(name.as_str(), value.to_str().unwrap_or(""));
        }

        // 复制 body
        let body = request.collect().await?.to_bytes();
        req_builder = req_builder.body(body);

        // 发送请求
        let response = req_builder.send().await?;

        Ok(response)
    }

    /// 转发并返回流式响应
    pub async fn forward_stream(
        &self,
        target_url: &str,
        _request: Request<hyper::body::Incoming>,
    ) -> Result<()> {
        // TODO: 实现流式转发
        tracing::info!("Forwarding stream to: {}", target_url);
        Ok(())
    }
}

impl Default for Forwarder {
    fn default() -> Self {
        Self::new()
    }
}