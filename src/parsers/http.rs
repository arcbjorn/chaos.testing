use crate::models::{RequestData, ResponseData};
use hyper::{HeaderMap, Method, Uri};
use std::collections::HashMap;

pub struct HttpParser;

impl HttpParser {
    pub fn parse_request(
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        body: Option<Vec<u8>>,
    ) -> RequestData {
        let headers_map = Self::headers_to_map(headers);
        let query_params = Self::parse_query_params(uri);

        RequestData {
            method: method.to_string(),
            uri: uri.to_string(),
            headers: headers_map,
            body,
            query_params,
        }
    }

    pub fn parse_response(
        status_code: u16,
        headers: &HeaderMap,
        body: Option<Vec<u8>>,
    ) -> ResponseData {
        ResponseData {
            status_code,
            headers: Self::headers_to_map(headers),
            body,
        }
    }

    fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect()
    }

    fn parse_query_params(uri: &Uri) -> HashMap<String, String> {
        uri.query()
            .map(|q| {
                q.split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        match (parts.next(), parts.next()) {
                            (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                            _ => None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn is_json_content(headers: &HeaderMap) -> bool {
        headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("application/json"))
            .unwrap_or(false)
    }

    pub fn extract_endpoint_pattern(uri: &Uri) -> String {
        let path = uri.path();

        path.split('/')
            .map(|segment| {
                if segment.chars().all(|c| c.is_numeric()) {
                    "{id}"
                } else if segment.len() > 20
                    && segment.chars().all(|c| c.is_alphanumeric() || c == '-')
                {
                    "{uuid}"
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }
}
