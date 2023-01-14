use std::ops::Add;

use crate::{split::SplitItem, options::TokenizerOptions};


#[derive(Debug, PartialEq, Clone)]
pub struct Token(pub TokenType, pub TokenData);

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    None,
    Any(String),
    /// @
    AtSign,
    /// //
    SingleLineComment,
    /// /*
    BeginMultilineComment,
    /// */
    EndMultilineComment,
    Comma,
    OpenParenthesis,
    CloseParenthesis,
    OpenCurlyBrace,
    CloseCurlyBrace,
    OpenBracket,
    CloseBracket,
    OpenSquareBracket,
    TrueLiteral,
    FalseLiteral,
    NullLiteral,
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(char),
    /// +
    Plus,
    /// -
    Minus,
    /// /
    Divide,
    /// *
    Multiply,
    /// %
    Modulo,
    Id(String),
    /// =
    Equal,
    /// ==
    EqualTo,
    ///!=
    NotEqual,
    /// <
    LessThan,
    /// <=
    LessThanOrEqual,
    /// >
    GreaterThan,
    /// >=
    GreaterThanOrEqual,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// &&
    And,
    /// ||
    Or,
    /// $
    DollarSign,
    /// ->
    Arrow,
    /// =>
    FatArrow,
    /// .
    Dot,
    /// #
    HashSign,
    /// !
    Exclamation,
    /// ?
    QuestionMark,
    EOF,
    End
}

impl Token {
    pub fn build(item: SplitItem, options: &TokenizerOptions) -> Token {
        let text = item.text;
        let data = TokenData::new(text.clone(), item.position);
        let mut token_type = match text.as_str() {
            "@" => TokenType::AtSign,
            "//" => TokenType::SingleLineComment,
            "/*" => TokenType::BeginMultilineComment,
            "*/" => TokenType::EndMultilineComment,
            "(" => TokenType::OpenParenthesis,
            ")" => TokenType::CloseParenthesis,
            "[" => TokenType::OpenBracket,
            "]" => TokenType::CloseBracket,
            "{" => TokenType::OpenCurlyBrace,
            "}" => TokenType::CloseCurlyBrace,
            "true" => TokenType::TrueLiteral,
            "false" => TokenType::FalseLiteral,
            "null" => TokenType::NullLiteral,
            "+" => TokenType::Plus,
            "-" => TokenType::Minus,
            "/" => TokenType::Divide,
            "*" => TokenType::Multiply,
            "%" => TokenType::Modulo,
            "=" => TokenType::Equal,
            "==" => TokenType::EqualTo,
            "!=" => TokenType::NotEqual,
            "<" => TokenType::LessThan,
            "<=" => TokenType::LessThanOrEqual,
            ">" => TokenType::GreaterThan,
            ">=" => TokenType::GreaterThanOrEqual,
            ":" => TokenType::Colon,
            ";" => TokenType::Semicolon,
            "&&" => TokenType::And,
            "||" => TokenType::Or,
            "$" => TokenType::DollarSign,
            "->" => TokenType::Arrow,
            "=>" => TokenType::FatArrow,
            "." => TokenType::Dot,
            "#" => TokenType::HashSign,
            "!" => TokenType::Exclamation,
            "?" => TokenType::QuestionMark,
            "," => TokenType::Comma,
            "\n" | "\r" => TokenType::EOF,
            _ => TokenType::None
        };
        if token_type == TokenType::None {
            if data.text.starts_with('"') && data.text.ends_with('"') {
                let mut fstr = data.text.to_owned();
                fstr.remove(0);
                fstr.remove(fstr.len() - 1);
                token_type = TokenType::StringLiteral(fstr);
            } else if (options.try_id)(data.text.as_str()) {
                token_type = TokenType::Id(data.text.to_owned());
            } else if let Ok(integer) = &data.text.parse::<i64>() {
                token_type = TokenType::IntegerLiteral(*integer);
            } else if data.text.ends_with('f') { 
                    if let Ok(float) = data.text[..data.text.len() - 1].parse::<f64>() {
                        token_type = TokenType::FloatLiteral(float);
                    } else { }
            } else if data.text.contains('.') {
                if let Ok(float) = data.text.parse::<f64>() {
                    token_type = TokenType::FloatLiteral(float);
                } else if let Ok(pcr) = data.text.parse::<char>() {
                    token_type = TokenType::CharLiteral(pcr);
                } else { }
            } else if data.text.starts_with('\'') && data.text.ends_with('\'') {
                let mut fcr = data.text.to_owned();
                fcr.remove(0);
                fcr.remove(fcr.len() - 1);
                if let Ok(pcr) = fcr.parse::<char>() {
                    token_type = TokenType::CharLiteral(pcr);
                } else { }
            }
        } else {  };
        Token(token_type, data)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenData {
    pub text: String,
    pub position: TokenPosition
}
impl TokenData {
    pub fn new(text: String, position: TokenPosition) -> TokenData {
        TokenData {
            text,
            position
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TokenPosition {
    pub position: i32,
    pub line: i32,
}
impl TokenPosition {
    pub fn new(position: i32, line: i32) -> TokenPosition {
        TokenPosition {
            position,
            line,
        }
    }
}
impl Add for TokenPosition {
    type Output = TokenPosition;
    fn add(self, other: TokenPosition) -> TokenPosition {
        TokenPosition {
            position: if self.position > other.position { self.position } else { other.position },
            line: if self.line > other.line { self.line } else { other.line }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::options::default;
    use super::*;

    #[test]
    fn test_build_token() {
        let item = SplitItem::new("test".to_owned(), TokenPosition::new(0, 0));
        let token = Token::build(item, &default());
        assert_eq!(token.0, TokenType::Id("test".to_owned()));
    }

    #[test]
    fn test_build_integer_literal_token() {
        let item = SplitItem::new("123".to_owned(), TokenPosition::new(0, 0));
        let token = Token::build(item, &default());
        assert_eq!(token.0, TokenType::IntegerLiteral(123));
    }

    #[test]
    fn test_build_float_literal_token() {
        let item = SplitItem::new("123.456".to_owned(), TokenPosition::new(0, 0));
        let token = Token::build(item, &default());
        assert_eq!(token.0, TokenType::FloatLiteral(123.456));
    }

    #[test]
    fn test_build_float_literal_token_with_f() {
        let item = SplitItem::new("123f".to_owned(), TokenPosition::new(0, 0));
        let token = Token::build(item, &default());
        assert_eq!(token.0, TokenType::FloatLiteral(123_f64));
    }

    #[test]
    fn test_build_char_literal_token() {
        let item = SplitItem::new("'a'".to_owned(), TokenPosition::new(0, 0));
        let token = Token::build(item, &default());
        assert_eq!(token.0, TokenType::CharLiteral('a'));
    }

    #[test]
    fn test_build_string_literal_token() {
        let item = SplitItem::new("\"test\"".to_owned(), TokenPosition::new(0, 0));
        let token = Token::build(item, &default());
        assert_eq!(token.0, TokenType::StringLiteral("test".to_owned()));
    }

    #[test]
    fn test_build_token_with_position() {
        let item = SplitItem::new("test".to_owned(), TokenPosition::new(1, 2));
        let token = Token::build(item, &default());
        assert_eq!(token.1.position.position, 1);
        assert_eq!(token.1.position.line, 2);
    }
}