//! Forwarder integration tests

use agw_core::core::Forwarder;
use agw_core::core::StreamForwardOptions;

#[test]
fn test_forwarder_new() {
    let _forwarder = Forwarder::new();
    // Just verify it creates successfully
    assert!(true);
}

#[test]
fn test_stream_forward_options_default() {
    let options = StreamForwardOptions::default();
    assert!(!options.convert_sse);
    assert!(options.conversion_type.is_none());
    assert_eq!(options.target_content_type, "text/event-stream");
}

#[test]
fn test_stream_forward_options_openai_to_anthropic() {
    let options = StreamForwardOptions::openai_to_anthropic();
    assert!(options.convert_sse);
    assert_eq!(options.conversion_type, Some("openai_to_anthropic".to_string()));
    assert_eq!(options.target_content_type, "text/event-stream");
}

#[test]
fn test_stream_forward_options_anthropic_to_openai() {
    let options = StreamForwardOptions::anthropic_to_openai();
    assert!(options.convert_sse);
    assert_eq!(options.conversion_type, Some("anthropic_to_openai".to_string()));
    assert_eq!(options.target_content_type, "text/event-stream");
}

#[test]
fn test_stream_forward_options_passthrough() {
    let options = StreamForwardOptions::passthrough();
    assert!(!options.convert_sse);
    assert!(options.conversion_type.is_none());
}

// Note: forward(), forward_stream(), forward_stream_with_options() require
// Request<hyper::body::Incoming> which is hard to construct in tests.
// These are tested via the API handler integration tests instead.

#[tokio::test]
async fn test_forwarder_convert_sse_stream() {
    use bytes::Bytes;
    use futures::stream;

    let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from("event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n")),
    ];

    let input_stream = stream::iter(chunks);
    let converted = Forwarder::convert_sse_stream(input_stream, "anthropic_to_openai");

    use futures::StreamExt;
    let results: Vec<_> = converted.collect().await;
    assert_eq!(results.len(), 1);

    let chunk = results[0].as_ref().unwrap();
    let text = String::from_utf8_lossy(chunk);
    assert!(text.contains("chat.completion.chunk"));
    assert!(text.contains("Hello"));
}

#[tokio::test]
async fn test_forwarder_convert_sse_stream_openai_to_anthropic() {
    use bytes::Bytes;
    use futures::stream;

    let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from("data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n")),
    ];

    let input_stream = stream::iter(chunks);
    let converted = Forwarder::convert_sse_stream(input_stream, "openai_to_anthropic");

    use futures::StreamExt;
    let results: Vec<_> = converted.collect().await;
    assert_eq!(results.len(), 1);

    let chunk = results[0].as_ref().unwrap();
    let text = String::from_utf8_lossy(chunk);
    assert!(text.contains("content_block_delta"));
    assert!(text.contains("Hello"));
}

#[tokio::test]
async fn test_forwarder_convert_sse_stream_passthrough() {
    use bytes::Bytes;
    use futures::stream;

    let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from("data: some data\n\n")),
    ];

    let input_stream = stream::iter(chunks);
    let converted = Forwarder::convert_sse_stream(input_stream, "unknown_conversion");

    use futures::StreamExt;
    let results: Vec<_> = converted.collect().await;
    assert_eq!(results.len(), 1);

    let chunk = results[0].as_ref().unwrap();
    let text = String::from_utf8_lossy(chunk);
    assert!(text.contains("some data"));
}

#[tokio::test]
async fn test_forwarder_convert_sse_stream_error_propagation() {
    use bytes::Bytes;
    use futures::stream;

    let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
        Err(std::io::Error::new(std::io::ErrorKind::Other, "test error")),
    ];

    let input_stream = stream::iter(chunks);
    let converted = Forwarder::convert_sse_stream(input_stream, "anthropic_to_openai");

    use futures::StreamExt;
    let results: Vec<_> = converted.collect().await;
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}
