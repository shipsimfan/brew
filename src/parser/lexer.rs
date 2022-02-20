use compiler::lexer::{CharIter, Token};

#[derive(Clone)]
pub enum TokenClass {
    EndOfFile,
    Newline,
    String(String),
    Comma,
    Equals,
}

fn tokenize_string(iter: &mut compiler::lexer::CharIter) -> Token<TokenClass> {
    let token_column = iter.column();
    let token_line = iter.line();

    let mut string = String::new();

    while let Some(c) = iter.next() {
        if !c.is_alphanumeric()
            && c != '_'
            && c != '.'
            && c != '-'
            && c != '+'
            && c != '/'
            && c != '\\'
            && c != '.'
        {
            iter.unget(c);
            break;
        }

        string.push(c);
    }

    Token::new(TokenClass::String(string), token_line, token_column)
}

pub fn get_next_token(
    iter: &mut CharIter,
) -> Result<Option<Token<TokenClass>>, Box<dyn std::error::Error>> {
    let c = iter.next().unwrap();

    if c.is_alphabetic() || c == '_' || c == '/' || c == '\\' || c == '.' {
        iter.unget(c);
        Ok(Some(tokenize_string(iter)))
    } else {
        match c {
            '#' => {
                // '#' start comments
                while match iter.next() {
                    Some(c) => c != '\n',
                    None => false,
                } {}

                Ok(None)
            }
            ',' => Ok(Some(Token::new(
                TokenClass::Comma,
                iter.line(),
                iter.column(),
            ))),
            '=' => Ok(Some(Token::new(
                TokenClass::Equals,
                iter.line(),
                iter.column(),
            ))),
            _ => Err(Box::new(super::error::BrewfileError::UnknownCharacter(
                c,
                iter.line(),
                iter.column(),
            ))),
        }
    }
}

impl compiler::lexer::TokenClass for TokenClass {}

impl std::fmt::Display for TokenClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenClass::EndOfFile => format!("End of file"),
                TokenClass::Newline => format!("Newline"),
                TokenClass::String(string) => format!("\"{}\"", string),
                TokenClass::Comma => format!("','"),
                TokenClass::Equals => format!("'='"),
            }
        )
    }
}
