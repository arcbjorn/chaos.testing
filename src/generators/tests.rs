use super::*;
use crate::models::{CapturedRequest, Protocol, RequestData, ResponseData};
use chrono::Utc;

fn create_test_request(method: &str, uri: &str, status: u16) -> CapturedRequest {
    CapturedRequest {
        id: "test-123".to_string(),
        timestamp: Utc::now(),
        protocol: Protocol::Http,
        request: RequestData {
            method: method.to_string(),
            uri: uri.to_string(),
            headers: Default::default(),
            body: Some(b"{\"test\":\"data\"}".to_vec()),
            query_params: Default::default(),
        },
        response: Some(ResponseData {
            status_code: status,
            headers: Default::default(),
            body: Some(b"{\"result\":\"ok\"}".to_vec()),
        }),
        duration_ms: Some(42),
    }
}

#[test]
fn test_python_generator() {
    let requests = vec![
        create_test_request("GET", "/api/users", 200),
        create_test_request("POST", "/api/users", 201),
    ];

    let generator = PythonGenerator;
    let code = generator.generate(&requests).unwrap();

    assert!(code.contains("import requests"));
    assert!(code.contains("def test_"));
    assert!(code.contains("GET"));
    assert!(code.contains("POST"));
    assert!(code.contains("/api/users"));
    assert!(code.contains("assert response.status_code == 200"));
}

#[test]
fn test_go_generator() {
    let requests = vec![
        create_test_request("GET", "/api/products", 200),
        create_test_request("DELETE", "/api/products/1", 204),
    ];

    let generator = GoGenerator;
    let code = generator.generate(&requests).unwrap();

    assert!(code.contains("package main"));
    assert!(code.contains("import"));
    assert!(code.contains("func Test"));
    assert!(code.contains("GET"));
    assert!(code.contains("DELETE"));
    assert!(code.contains("/api/products"));
    assert!(code.contains("if resp.StatusCode != 200"));
}

#[test]
fn test_rust_generator() {
    let requests = vec![
        create_test_request("PUT", "/api/orders/1", 200),
        create_test_request("PATCH", "/api/orders/1", 200),
    ];

    let generator = RustGenerator;
    let code = generator.generate(&requests).unwrap();

    assert!(code.contains("use reqwest"));
    assert!(code.contains("#[tokio::test]"));
    assert!(code.contains("async fn test_"));
    assert!(code.contains("PUT"));
    assert!(code.contains("PATCH"));
    assert!(code.contains("/api/orders"));
    assert!(code.contains("assert_eq!(response.status().as_u16(), 200)"));
}

#[test]
fn test_get_generator_auto_detection() {
    let result = get_generator("auto", None);
    assert!(result.is_ok());
}

#[test]
fn test_get_generator_python() {
    let result = get_generator("python", None);
    assert!(result.is_ok());
}

#[test]
fn test_get_generator_go() {
    let result = get_generator("go", None);
    assert!(result.is_ok());
}

#[test]
fn test_get_generator_rust() {
    let result = get_generator("rust", None);
    assert!(result.is_ok());
}

#[test]
fn test_get_generator_unknown() {
    let result = get_generator("unknown_language", None);
    assert!(result.is_err());
}

#[test]
fn test_empty_requests() {
    let requests = vec![];
    let generator = PythonGenerator;
    let code = generator.generate(&requests).unwrap();
    assert!(code.contains("import requests"));
}

#[test]
fn test_multiple_methods_same_endpoint() {
    let requests = vec![
        create_test_request("GET", "/api/users", 200),
        create_test_request("POST", "/api/users", 201),
        create_test_request("PUT", "/api/users", 200),
        create_test_request("DELETE", "/api/users", 204),
    ];

    let generator = PythonGenerator;
    let code = generator.generate(&requests).unwrap();

    assert!(code.matches("def test_").count() >= 4);
}
