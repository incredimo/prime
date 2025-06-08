// src/web_ops.rs
use anyhow::{Context as AnyhowContext, Result, anyhow};
use reqwest::Client;
use futures::StreamExt; // For byte_stream().next()
use log; // For logging

const MAX_RESPONSE_BYTES: u64 = 2_000_000; // 2MB limit for fetched content
const REQUEST_TIMEOUT_SECONDS: u64 = 30;

pub async fn handle_fetch_text_web_op(
    http_client: &Client,
    url: &str,
) -> Result<String> {
    if url.trim().is_empty() {
        return Err(anyhow!("URL cannot be empty."));
    }

    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
       return Err(anyhow!("Invalid URL scheme: URL must start with http:// or https://"));
    }

    log::info!("Fetching text from URL: {}", url);

    let response = http_client
        .get(url)
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await
        .with_context(|| format!("Failed to send request to URL: {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Request to {} failed with status: {}",
            url,
            response.status()
        ));
    }

    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        if let Ok(ct_str) = content_type.to_str() {
            log::debug!("Content-Type for {}: {}", url, ct_str);
            if ct_str.starts_with("image/")
                || ct_str.starts_with("audio/")
                || ct_str.starts_with("video/")
                || ct_str.starts_with("application/pdf")
                || ct_str == "application/octet-stream"
            {
                return Err(anyhow!(
                    "URL points to binary or non-text content ({}). Only text-based content can be fetched.",
                    ct_str
                ));
            }
        }
    }

    let mut byte_stream = response.bytes_stream();
    let mut buffer = Vec::new();
    let mut total_bytes_read: u64 = 0;
    let mut truncated = false;

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = chunk_result.with_context(|| format!("Error reading stream from URL: {}", url))?;
        if total_bytes_read.saturating_add(chunk.len() as u64) > MAX_RESPONSE_BYTES {
            let remaining_space = MAX_RESPONSE_BYTES.saturating_sub(total_bytes_read);
            if remaining_space > 0 {
               buffer.extend_from_slice(&chunk[..(remaining_space as usize)]);
               total_bytes_read = total_bytes_read.saturating_add(remaining_space);
            }
            truncated = true;
            log::warn!("Response from {} truncated at {} bytes.", url, MAX_RESPONSE_BYTES);
            break;
        }
        buffer.extend_from_slice(&chunk);
        total_bytes_read = total_bytes_read.saturating_add(chunk.len() as u64);
    }

    if buffer.is_empty() && total_bytes_read == 0 && !truncated {
        log::info!("Response from {} was successful but empty.", url);
        // Return empty string for successful empty responses.
    }

    let mut text_content = String::from_utf8(buffer)
        .with_context(|| format!("Failed to decode response from {} as UTF-8. Content might be non-text or use an unsupported encoding.", url))?;

    if truncated {
        text_content.push_str("\n\n... (content truncated due to size limit)");
    }

    log::info!("Successfully fetched and processed content from {}. Size: {} bytes. Truncated: {}", url, total_bytes_read, truncated);
    Ok(text_content)
}

#[cfg(test)]
mod tests {
    use super::*; // Imports handle_fetch_text_web_op and constants
    use httpmock::prelude::*; // Imports MockServer and related features
    use reqwest::Client; // To create a client for tests

    fn create_test_client() -> Client {
        // Create a basic client for tests.
        Client::builder().build().unwrap()
    }

    #[tokio::test]
    async fn test_fetch_successful() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/success");

        let expected_body = "Hello, world!";
        server.mock(|when, then| {
            when.method(GET).path("/success");
            then.status(200)
                .header("Content-Type", "text/plain")
                .body(expected_body);
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_body);
    }

    #[tokio::test]
    async fn test_fetch_http_404_error() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/notfound");

        server.mock(|when, then| {
            when.method(GET).path("/notfound");
            then.status(404);
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("failed with status: 404 Not Found"));
    }

    #[tokio::test]
    async fn test_fetch_http_500_error() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/servererror");

        server.mock(|when, then| {
            when.method(GET).path("/servererror");
            then.status(500);
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("failed with status: 500 Internal Server Error"));
    }

    #[tokio::test]
    async fn test_fetch_network_error() {
        let client = create_test_client();
        let non_existent_url = "http://localhost:12345/nonexistent";

        let result = handle_fetch_text_web_op(&client, non_existent_url).await;
        assert!(result.is_err());
        // Check that the error message indicates a failure to send the request or a connection problem
        let err_string = result.err().unwrap().to_string().to_lowercase();
        assert!(err_string.contains("failed to send request") || err_string.contains("connection refused") || err_string.contains("dns error") || err_string.contains("error sending request"));
    }

    #[tokio::test]
    async fn test_fetch_timeout() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/timeout");

        server.mock(|when, then| {
            when.method(GET).path("/timeout");
            then.status(200)
                .delay(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECONDS + 1)); // Delay longer than handler's timeout
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_err());
        let err_string = result.err().unwrap().to_string().to_lowercase();
        assert!(err_string.contains("timed out") || err_string.contains("timeout"));
    }

    #[tokio::test]
    async fn test_fetch_content_truncation() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/large");

        let large_body_segment = "a".repeat(1024 * 1024); // 1MB
        let body_content = format!("{}{}", large_body_segment, large_body_segment.repeat(2)); // Approx 3MB

        server.mock(|when, then| {
            when.method(GET).path("/large");
            then.status(200)
                .header("Content-Type", "text/plain")
                .body(&body_content);
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_ok());
        let fetched_text = result.unwrap();
        // Expected length is MAX_RESPONSE_BYTES + length of truncation message
        let expected_max_len = (MAX_RESPONSE_BYTES as usize) + "\n\n... (content truncated due to size limit)".len();
        assert!(fetched_text.len() <= expected_max_len);
        assert!(fetched_text.ends_with("\n\n... (content truncated due to size limit)"));
        // Check if the beginning of the content matches the original, up to MAX_RESPONSE_BYTES
        assert!(body_content.starts_with(&fetched_text[..(MAX_RESPONSE_BYTES as usize)]));
    }

    #[tokio::test]
    async fn test_fetch_binary_content_rejection_image() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/image");

        server.mock(|when, then| {
            when.method(GET).path("/image");
            then.status(200).header("Content-Type", "image/jpeg");
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("points to binary or non-text content (image/jpeg)"));
    }

    #[tokio::test]
    async fn test_fetch_binary_content_rejection_pdf() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/pdf");

        server.mock(|when, then| {
            when.method(GET).path("/pdf");
            then.status(200).header("Content-Type", "application/pdf");
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("points to binary or non-text content (application/pdf)"));
    }

    #[tokio::test]
    async fn test_fetch_empty_url_error() {
        let client = create_test_client();
        let result = handle_fetch_text_web_op(&client, "").await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("URL cannot be empty"));
    }

    #[tokio::test]
    async fn test_fetch_invalid_scheme_error() {
        let client = create_test_client();
        let result = handle_fetch_text_web_op(&client, "ftp://example.com").await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Invalid URL scheme"));
    }

    #[tokio::test]
    async fn test_fetch_successful_empty_response() {
        let server = MockServer::start();
        let client = create_test_client();
        let mock_url = server.url("/empty_success");

        server.mock(|when, then| {
            when.method(GET).path("/empty_success");
            then.status(200)
                .header("Content-Type", "text/plain")
                .body(""); // Empty body
        });

        let result = handle_fetch_text_web_op(&client, &mock_url).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
