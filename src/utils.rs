use crate::parsers::redis::{RedisCommandType, RedisParser};
use crate::parsers::sql::{QueryType, SqlParser};

#[allow(dead_code)]
pub struct QueryAnalyzer;

impl QueryAnalyzer {
    #[allow(dead_code)]
    pub fn analyze_sql_query(query: &str) -> (QueryType, bool) {
        let query_type = SqlParser::classify_query(query);
        let is_read_only = matches!(query_type, QueryType::Select);
        (query_type, is_read_only)
    }

    #[allow(dead_code)]
    pub fn analyze_redis_command(command: &str) -> (RedisCommandType, bool) {
        let cmd_type = RedisParser::classify_command(command);
        let is_read_only = RedisParser::is_read_only(command);
        (cmd_type, is_read_only)
    }

    #[allow(dead_code)]
    pub fn is_safe_operation(query: &str) -> bool {
        if query.to_uppercase().starts_with("SELECT") {
            let (_, is_read_only) = Self::analyze_sql_query(query);
            return is_read_only;
        }

        if let Some(first_word) = query.split_whitespace().next() {
            let (_, is_read_only) = Self::analyze_redis_command(first_word);
            return is_read_only;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_classification() {
        let (qtype, read_only) = QueryAnalyzer::analyze_sql_query("SELECT * FROM users");
        assert_eq!(qtype, QueryType::Select);
        assert!(read_only);

        let (qtype, read_only) = QueryAnalyzer::analyze_sql_query("INSERT INTO users VALUES (1)");
        assert_eq!(qtype, QueryType::Insert);
        assert!(!read_only);
    }

    #[test]
    fn test_redis_classification() {
        let (cmd_type, read_only) = QueryAnalyzer::analyze_redis_command("GET");
        assert_eq!(cmd_type, RedisCommandType::Read);
        assert!(read_only);

        let (cmd_type, read_only) = QueryAnalyzer::analyze_redis_command("SET");
        assert_eq!(cmd_type, RedisCommandType::Write);
        assert!(!read_only);
    }

    #[test]
    fn test_safe_operations() {
        assert!(QueryAnalyzer::is_safe_operation("SELECT * FROM users"));
        assert!(!QueryAnalyzer::is_safe_operation("DELETE FROM users"));
    }
}
