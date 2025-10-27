use crate::models::RedisCommand;

#[allow(dead_code)]
pub struct RedisParser;

impl RedisParser {
    /// Parse RESP (Redis Serialization Protocol)
    #[allow(dead_code)]
    pub fn parse(data: &[u8]) -> Option<RedisCommand> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            b'*' => Self::parse_array(data),
            b'$' => Self::parse_bulk_string(data).map(|cmd| RedisCommand {
                command: cmd,
                args: vec![],
                database: 0,
            }),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn parse_array(data: &[u8]) -> Option<RedisCommand> {
        let lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();
        if lines.is_empty() {
            return None;
        }

        let mut parts = Vec::new();
        let mut i = 1;

        while i < lines.len() {
            if i + 1 < lines.len() && !lines[i + 1].is_empty() {
                if let Ok(s) = std::str::from_utf8(lines[i + 1]) {
                    parts.push(s.trim_end_matches('\r').to_string());
                }
                i += 2;
            } else {
                break;
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(RedisCommand {
            command: parts[0].clone().to_uppercase(),
            args: parts.into_iter().skip(1).collect(),
            database: 0,
        })
    }

    #[allow(dead_code)]
    fn parse_bulk_string(data: &[u8]) -> Option<String> {
        let s = std::str::from_utf8(data).ok()?;
        let lines: Vec<&str> = s.lines().collect();
        if lines.len() < 2 {
            return None;
        }
        Some(lines[1].to_string())
    }

    #[allow(dead_code)]
    pub fn classify_command(command: &str) -> RedisCommandType {
        match command.to_uppercase().as_str() {
            "GET" | "MGET" | "HGET" | "HGETALL" | "LRANGE" | "SMEMBERS" | "ZRANGE" => {
                RedisCommandType::Read
            }
            "SET" | "MSET" | "HSET" | "LPUSH" | "RPUSH" | "SADD" | "ZADD" => {
                RedisCommandType::Write
            }
            "DEL" | "HDEL" | "LPOP" | "RPOP" | "SREM" | "ZREM" => RedisCommandType::Delete,
            "INCR" | "DECR" | "INCRBY" | "DECRBY" | "HINCRBY" => RedisCommandType::Increment,
            "EXPIRE" | "TTL" | "PERSIST" => RedisCommandType::Expiry,
            "PING" | "ECHO" | "INFO" => RedisCommandType::Admin,
            _ => RedisCommandType::Other,
        }
    }

    #[allow(dead_code)]
    pub fn is_read_only(command: &str) -> bool {
        matches!(
            Self::classify_command(command),
            RedisCommandType::Read | RedisCommandType::Admin
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisCommandType {
    Read,
    Write,
    Delete,
    Increment,
    Expiry,
    Admin,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_commands() {
        assert_eq!(RedisParser::classify_command("GET"), RedisCommandType::Read);
        assert_eq!(
            RedisParser::classify_command("SET"),
            RedisCommandType::Write
        );
        assert!(RedisParser::is_read_only("GET"));
        assert!(!RedisParser::is_read_only("SET"));
    }
}
