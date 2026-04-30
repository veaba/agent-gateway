//! 请求转发器

use anyhow::Result;
use bytes::Bytes;
use futures::Stream;
use reqwest::Client;
use hyper::Request;
use http_body_util::BodyExt;
use std::pin::Pin;
use futures_util::StreamExt;

/// SSE 流式转发选项
#[derive(Debug, Clone)]
pub struct StreamForwardOptions {
    /// 是否转换 SSE 格式
    pub convert_sse: bool,
    /// 转换类型 (openai_to_anthropic / anthropic_to_openai)
    pub conversion_type: Option<String>,
    /// 目标 content-type
    pub target_content_type: String,
}

impl Default for StreamForwardOptions {
    fn default() -> Self {
        Self {
            convert_sse: false,
            conversion_type: None,
            target_content_type: "text/event-stream".to_string(),
        }
    }
}

impl StreamForwardOptions {
    /// 创建 OpenAI 到 Anthropic 的转换选项
    pub fn openai_to_anthropic() -> Self {
        Self {
            convert_sse: true,
            conversion_type: Some("openai_to_anthropic".to_string()),
            target_content_type: "text/event-stream".to_string(),
        }
    }

    /// 创建 Anthropic 到 OpenAI 的转换选项
    pub fn anthropic_to_openai() -> Self {
        Self {
            convert_sse: true,
            conversion_type: Some("anthropic_to_openai".to_string()),
            target_content_type: "text/event-stream".to_string(),
        }
    }

    /// 创建透传选项（不做转换）
    pub fn passthrough() -> Self {
        Self::default()
    }
}

/// 流式响应的字节流类型
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

/// 请求转发器
pub struct Forwarder {
    /// HTTP 客户端
    pub client: Client,
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

    /// 转发并返回流式响应（不做 SSE 转换，直接透传）
    pub async fn forward_stream(
        &self,
        target_url: &str,
        request: Request<hyper::body::Incoming>,
    ) -> Result<reqwest::Response> {
        self.forward_stream_with_options(target_url, request, StreamForwardOptions::passthrough()).await
    }

    /// 转发并返回流式响应（带 SSE 转换选项）
    pub async fn forward_stream_with_options(
        &self,
        target_url: &str,
        request: Request<hyper::body::Incoming>,
        options: StreamForwardOptions,
    ) -> Result<reqwest::Response> {
        tracing::debug!("Forwarding stream to: {} (convert: {})", target_url, options.convert_sse);

        // 构建转发的请求
        let mut req_builder = self.client
            .request(
                reqwest::Method::from_bytes(request.method().as_str().as_bytes()).unwrap(),
                target_url,
            );

        // 复制 headers (跳过 content-length)
        for (name, value) in request.headers() {
            let name_str = name.as_str();
            if name_str != "content-length" {
                if let Ok(v) = value.to_str() {
                    req_builder = req_builder.header(name_str, v);
                }
            }
        }

        // 确保启用流式响应
        req_builder = req_builder.header("Accept", "text/event-stream");

        // 复制 body
        let body = request.collect().await?.to_bytes();
        req_builder = req_builder.body(body);

        // 发送请求
        let response = req_builder.send().await?;

        Ok(response)
    }

    /// 转换 SSE 流 (静态方法)
    pub fn convert_sse_stream<S, E>(
        stream: S,
        conversion_type: &str,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, E>> + Send>>
    where
        S: Stream<Item = Result<Bytes, E>> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let conversion_type = conversion_type.to_string();

        let converted = stream.map(move |chunk: Result<Bytes, E>| {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    tracing::trace!("SSE chunk ({} bytes): {:?}", text.len(), &text[..text.len().min(200)]);

                    // 按行处理 SSE 数据
                    let converted: String = text
                        .lines()
                        .filter_map(|line| {
                            if conversion_type == "anthropic_to_openai" {
                                crate::core::converter::anthropic_sse_to_openai(line)
                            } else if conversion_type == "openai_to_anthropic" {
                                crate::core::converter::openai_sse_to_anthropic(line)
                            } else {
                                Some(line.to_string())
                            }
                        })
                        .collect();

                    if converted.is_empty() {
                        Ok(Bytes::new())
                    } else {
                        Ok(Bytes::from(converted))
                    }
                }
                Err(e) => Err(e),
            }
        });

        Box::pin(converted)
    }
}

impl Default for Forwarder {
    fn default() -> Self {
        Self::new()
    }
}