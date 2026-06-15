//! Rule expression lexer.

use crate::rules::error::RuleError;

/// Token emitted by the rule lexer.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// Identifier or dotted variable path.
    Ident(String),
    /// String literal.
    String(String),
    /// Number literal.
    Number(f64),
    /// Boolean literal.
    Bool(bool),
    /// Null literal.
    Null,
    /// Symbol/operator.
    Symbol(String),
    /// End of input.
    Eof,
}

/// Tokenizes a rule expression.
///
/// # Errors
///
/// Returns an error for invalid strings or number units.
pub fn lex(input: &str) -> Result<Vec<Token>, RuleError> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(character) = chars.peek().copied() {
        match character {
            ' ' | '\t' | '\r' | '\n' => {
                chars.next();
            }
            '"' => tokens.push(Token::String(read_string(&mut chars)?)),
            '0'..='9' => tokens.push(Token::Number(read_number(&mut chars)?)),
            '[' | ']' | '(' | ')' | ',' | '+' | '-' | '*' | '/' => {
                tokens.push(Token::Symbol(character.to_string()));
                chars.next();
            }
            '!' | '=' | '>' | '<' | '&' | '|' => {
                tokens.push(Token::Symbol(read_operator(&mut chars)?));
            }
            _ if is_ident_start(character) => {
                let ident = read_ident(&mut chars);
                tokens.push(match ident.as_str() {
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    "null" => Token::Null,
                    "in" | "contains" | "starts_with" | "ends_with" | "matches" => {
                        Token::Symbol(ident)
                    }
                    _ => Token::Ident(ident),
                });
            }
            _ => {
                return Err(RuleError::new(format!(
                    "unexpected character '{character}'"
                )));
            }
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

fn read_string(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
) -> Result<String, RuleError> {
    chars.next();
    let mut output = String::new();
    while let Some(character) = chars.next() {
        match character {
            '"' => return Ok(output),
            '\\' => {
                if let Some(escaped) = chars.next() {
                    output.push(escaped);
                }
            }
            _ => output.push(character),
        }
    }
    Err(RuleError::new("unterminated string literal"))
}

fn read_number(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
) -> Result<f64, RuleError> {
    let mut raw = String::new();
    while chars
        .peek()
        .is_some_and(|character| character.is_ascii_digit() || *character == '.')
    {
        raw.push(chars.next().expect("peeked character"));
    }
    let mut unit = String::new();
    while chars.peek().is_some_and(char::is_ascii_alphabetic) {
        unit.push(chars.next().expect("peeked character"));
    }
    let number = raw
        .parse::<f64>()
        .map_err(|_| RuleError::new(format!("invalid number '{raw}'")))?;
    Ok(number * unit_multiplier(&unit)?)
}

fn unit_multiplier(unit: &str) -> Result<f64, RuleError> {
    match unit {
        "" | "s" => Ok(1.0),
        "kb" => Ok(1_024.0),
        "mb" => Ok(1_048_576.0),
        "gb" => Ok(1_073_741_824.0),
        "ms" => Ok(0.001),
        "m" => Ok(60.0),
        "h" => Ok(3_600.0),
        _ => Err(RuleError::new(format!("unsupported unit '{unit}'"))),
    }
}

fn read_operator(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
) -> Result<String, RuleError> {
    let first = chars.next().expect("operator first char");
    let second = chars.peek().copied();
    let operator = match (first, second) {
        ('=' | '!' | '>' | '<', Some('=')) => {
            chars.next();
            format!("{first}=")
        }
        ('&', Some('&')) | ('|', Some('|')) => {
            chars.next();
            format!("{first}{first}")
        }
        ('!' | '>' | '<', _) => first.to_string(),
        _ => return Err(RuleError::new(format!("invalid operator '{first}'"))),
    };
    Ok(operator)
}

fn read_ident(chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> String {
    let mut ident = String::new();
    while chars.peek().is_some_and(|character| {
        character.is_ascii_alphanumeric() || *character == '_' || *character == '.'
    }) {
        ident.push(chars.next().expect("peeked character"));
    }
    ident
}

fn is_ident_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}
