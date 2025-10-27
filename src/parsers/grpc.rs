//! gRPC request parser
//!
//! Parses gRPC requests and extracts service, method, and metadata.

/// gRPC request representation
#[derive(Debug, Clone)]
pub struct GrpcRequest {
    pub service: String,
    pub method: String,
    pub metadata: Vec<(String, String)>,
}

/// gRPC parser
pub struct GrpcParser;

impl GrpcParser {
    /// Parse a gRPC service path
    ///
    /// # Examples
    ///
    /// ```
    /// use chaos_testing::parsers::grpc::GrpcParser;
    ///
    /// if let Some((service, method)) = GrpcParser::parse_service_path("/users.UserService/GetUser") {
    ///     assert_eq!(service, "users.UserService");
    ///     assert_eq!(method, "GetUser");
    /// }
    /// ```
    pub fn parse_service_path(path: &str) -> Option<(String, String)> {
        if !path.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = path[1..].split('/').collect();
        if parts.len() != 2 {
            return None;
        }

        Some((parts[0].to_string(), parts[1].to_string()))
    }

    /// Extract package name from service
    pub fn extract_package(service: &str) -> Option<String> {
        service.rsplit_once('.').map(|(pkg, _)| pkg.to_string())
    }

    /// Classify gRPC method by naming pattern
    pub fn classify_method(method: &str) -> MethodType {
        let method_lower = method.to_lowercase();
        if method_lower.starts_with("get") || method_lower.starts_with("list") {
            MethodType::Query
        } else if method_lower.starts_with("create") || method_lower.starts_with("add") {
            MethodType::Create
        } else if method_lower.starts_with("update") || method_lower.starts_with("modify") {
            MethodType::Update
        } else if method_lower.starts_with("delete") || method_lower.starts_with("remove") {
            MethodType::Delete
        } else if method_lower.starts_with("watch") || method_lower.contains("stream") {
            MethodType::Stream
        } else {
            MethodType::Unary
        }
    }

    /// Check if method is streaming
    pub fn is_streaming(method: &str) -> bool {
        matches!(Self::classify_method(method), MethodType::Stream)
    }
}

/// gRPC method classification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodType {
    Query,
    Create,
    Update,
    Delete,
    Stream,
    Unary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service_path() {
        let (service, method) =
            GrpcParser::parse_service_path("/users.UserService/GetUser").unwrap();
        assert_eq!(service, "users.UserService");
        assert_eq!(method, "GetUser");
    }

    #[test]
    fn test_extract_package() {
        assert_eq!(
            GrpcParser::extract_package("users.UserService"),
            Some("users".to_string())
        );
    }

    #[test]
    fn test_classify_method() {
        assert_eq!(GrpcParser::classify_method("GetUser"), MethodType::Query);
        assert_eq!(
            GrpcParser::classify_method("CreateUser"),
            MethodType::Create
        );
        assert_eq!(
            GrpcParser::classify_method("UpdateUser"),
            MethodType::Update
        );
        assert_eq!(
            GrpcParser::classify_method("DeleteUser"),
            MethodType::Delete
        );
        assert_eq!(
            GrpcParser::classify_method("WatchUsers"),
            MethodType::Stream
        );
    }

    #[test]
    fn test_is_streaming() {
        assert!(GrpcParser::is_streaming("WatchUsers"));
        assert!(!GrpcParser::is_streaming("GetUser"));
    }
}
