#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
  Int,
  Double,
  Str,
  Char,
  Bool,
  Identifier,
  Keyword,
  Symbol,
  Operator,
  EOL,
  EOF,
}



#[derive(Debug, Clone, PartialEq)]
pub struct Token<'t> {
  pub token_type: TokenType,
  pub line:       usize,
  pub slice:      (usize, usize),
  pub lexeme:     String,

  lines: &'t Vec<String>,
}

impl<'t> Token<'t> {
  pub fn new(token_type: TokenType, line: usize, slice: (usize, usize), lexeme: &'t str, lines: &'t Vec<String>) -> Self {
    Token {
      token_type,
      line,
      slice,
      lexeme,
      lines
    }
  }
}