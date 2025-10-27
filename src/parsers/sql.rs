use crate::models::SqlQuery;
use regex::Regex;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub struct SqlParser;

impl SqlParser {
    pub fn parse(query: &str) -> Option<SqlQuery> {
        let dialect = GenericDialect {};

        match Parser::parse_sql(&dialect, query) {
            Ok(statements) if !statements.is_empty() => Some(SqlQuery {
                query: query.to_string(),
                params: Self::extract_params(query),
                database: None,
            }),
            _ => None,
        }
    }

    #[allow(clippy::if_same_then_else)]
    pub fn classify_query(query: &str) -> QueryType {
        let query_upper = query.trim().to_uppercase();

        if query_upper.starts_with("SELECT") {
            QueryType::Select
        } else if query_upper.starts_with("INSERT") {
            QueryType::Insert
        } else if query_upper.starts_with("UPDATE") {
            QueryType::Update
        } else if query_upper.starts_with("DELETE") {
            QueryType::Delete
        } else if query_upper.starts_with("CREATE") {
            QueryType::Ddl
        } else if query_upper.starts_with("ALTER") {
            QueryType::Ddl
        } else if query_upper.starts_with("DROP") {
            QueryType::Ddl
        } else {
            QueryType::Other
        }
    }

    pub fn extract_table_names(query: &str) -> Vec<String> {
        let dialect = GenericDialect {};

        match Parser::parse_sql(&dialect, query) {
            Ok(statements) => statements
                .into_iter()
                .flat_map(|stmt| Self::tables_from_statement(&stmt))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn tables_from_statement(stmt: &Statement) -> Vec<String> {
        match stmt {
            Statement::Query(query) => {
                let mut tables = Vec::new();
                if let Some(body) = &query.body.as_select() {
                    for table in &body.from {
                        if let Some(name) = &table.relation.to_string().split('.').next_back() {
                            tables.push(name.to_string());
                        }
                    }
                }
                tables
            }
            Statement::Insert(insert) => {
                vec![insert.table.to_string()]
            }
            Statement::Update { table, .. } => {
                vec![table.to_string()]
            }
            Statement::Delete(delete) => delete.tables.iter().map(|t| t.to_string()).collect(),
            _ => Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn extract_params(query: &str) -> Vec<String> {
        let param_regex = Regex::new(r"\$\d+|\?|:\w+").unwrap();
        param_regex
            .find_iter(query)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Ddl,
    Other,
}
