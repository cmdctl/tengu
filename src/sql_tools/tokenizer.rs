use std::collections::HashSet;

use super::tools::Table;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Token {
    Number(String),
    Token(String),
    Ignored(String),
}
pub fn tokenize(sql: String) -> HashSet<Token> {
    let mut tokens = HashSet::new();
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                continue;
            }
            '0'..='9' => {
                let mut number = String::new();
                number.push(c);
                while let Some(&c) = chars.peek() {
                    match c {
                        '0'..='9' => {
                            number.push(c);
                            chars.next();
                        }
                        _ => {
                            break;
                        }
                    }
                }
                tokens.insert(Token::Number(number));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut token = String::new();
                token.push(c);
                while let Some(&c) = chars.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            token.push(c);
                            chars.next();
                        }
                        _ => {
                            break;
                        }
                    }
                }
                tokens.insert(Token::Token(token));
            }
            _ => {
                tokens.insert(Token::Ignored(c.to_string()));
            }
        }
    }
    tokens
}

impl FromIterator<Token> for HashSet<Table> {
    fn from_iter<I: IntoIterator<Item = Token>>(iter: I) -> Self {
        let mut tables = HashSet::new();
        for token in iter {
            match token {
                Token::Token(token) => {
                    tables.insert(Table {
                        name: token,
                        schema: "".to_string(),
                    });
                }
                _ => {}
            }
        }
        tables
    }
}

pub fn intersection(list1: Vec<String>, list2: Vec<String>) -> Vec<String> {
    let set1: std::collections::HashSet<String> = list1.iter().cloned().collect();
    let set2: std::collections::HashSet<String> = list2.iter().cloned().collect();
    set1.intersection(&set2).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tokenize_sql_query() {
        let sql = "SELECT * FROM users WHERE id = 1".to_string();
        let tokens = tokenize(sql);
        assert_eq!(tokens.len(), 8);
        assert!(tokens.contains(&Token::Token("SELECT".to_string())));
        assert!(tokens.contains(&Token::Ignored("*".to_string())));
        assert!(tokens.contains(&Token::Token("FROM".to_string())));
        assert!(tokens.contains(&Token::Token("users".to_string())));
        assert!(tokens.contains(&Token::Token("WHERE".to_string())));
        assert!(tokens.contains(&Token::Token("id".to_string())));
        assert!(tokens.contains(&Token::Ignored("=".to_string())));
        assert!(tokens.contains(&Token::Number("1".to_string())));
    }
}
