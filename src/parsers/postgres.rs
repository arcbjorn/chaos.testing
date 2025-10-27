use crate::models::SqlQuery;

#[allow(dead_code)]
pub struct PostgresParser;

impl PostgresParser {
    /// Parse PostgreSQL wire protocol message
    #[allow(dead_code)]
    pub fn parse_simple_query(data: &[u8]) -> Option<SqlQuery> {
        if data.len() < 5 {
            return None;
        }

        if data[0] != b'Q' {
            return None;
        }

        let length = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;

        if data.len() < 5 + length - 4 {
            return None;
        }

        let query_bytes = &data[5..5 + length - 4];
        let query = String::from_utf8_lossy(query_bytes)
            .trim_end_matches('\0')
            .to_string();

        Some(SqlQuery {
            query,
            params: vec![],
            database: Some("postgres".to_string()),
        })
    }

    /// Parse prepared statement (Parse message)
    #[allow(dead_code)]
    pub fn parse_prepared_statement(data: &[u8]) -> Option<(String, SqlQuery)> {
        if data.len() < 5 || data[0] != b'P' {
            return None;
        }

        let mut pos = 5;
        let stmt_name_end = data[pos..].iter().position(|&b| b == 0)?;
        let stmt_name = String::from_utf8_lossy(&data[pos..pos + stmt_name_end]).to_string();

        pos += stmt_name_end + 1;
        let query_end = data[pos..].iter().position(|&b| b == 0)?;
        let query = String::from_utf8_lossy(&data[pos..pos + query_end]).to_string();

        Some((
            stmt_name,
            SqlQuery {
                query,
                params: vec![],
                database: Some("postgres".to_string()),
            },
        ))
    }

    #[allow(dead_code)]
    pub fn message_type(data: &[u8]) -> Option<PostgresMessageType> {
        if data.is_empty() {
            return None;
        }

        Some(match data[0] {
            b'Q' => PostgresMessageType::SimpleQuery,
            b'P' => PostgresMessageType::Parse,
            b'B' => PostgresMessageType::Bind,
            b'E' => PostgresMessageType::Execute,
            b'D' => PostgresMessageType::Describe,
            b'S' => PostgresMessageType::Sync,
            b'X' => PostgresMessageType::Terminate,
            b'C' => PostgresMessageType::Close,
            _ => PostgresMessageType::Unknown,
        })
    }

    #[allow(dead_code)]
    pub fn extract_table_names(query: &str) -> Vec<String> {
        use crate::parsers::sql::SqlParser;
        SqlParser::extract_table_names(query)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostgresMessageType {
    SimpleQuery,
    Parse,
    Bind,
    Execute,
    Describe,
    Sync,
    Terminate,
    Close,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(
            PostgresParser::message_type(b"Q"),
            Some(PostgresMessageType::SimpleQuery)
        );
        assert_eq!(
            PostgresParser::message_type(b"P"),
            Some(PostgresMessageType::Parse)
        );
    }
}
