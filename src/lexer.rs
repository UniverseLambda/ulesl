use std::io::{BufReader, Read};

use crate::common::Location;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
	IntegerLiteral,
	StringLiteral,
	BoolLiteral,
	Keyword,
	Identifier,
	SpecialInstruction,
	Operator,
}

#[derive(Clone, Debug)]
pub struct Token {
	pub token_type: TokenType,
	pub content: String,
	pub location: Location,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	// Unknown,
	#[error("Internal error")]
	Internal,
	#[error("End of file")]
	EndOfFile,
	#[error("{0}: Unexpected EOF")]
	UnexpectedEndOfFile(Location),
	#[error("{0}: Decoder error")]
	Decoder(Location),
	#[error("{0}: Invalid code point")]
	InvalidCodePoint(Location),
	#[error("{0}: Invalid character: {1:?}")]
	InvalidCharacter(Location, char),
	#[error("{0}: UnknownSpecialInstruction: {1}")]
	UnknownSpecialInstruction(Location, String),
}

enum LexerMode {
	// Error,
	Word,
	Number,
	String(bool, bool, bool),
	Operator,
}

pub struct Lexer<T: Read> {
	// source: String,
	reader: BufReader<T>,
	curr_char: char,
	buffered_char: Option<char>,
	curr_location: Location,
	line: usize,
	col: usize,
}

impl<T> Lexer<T>
where
	T: Read,
{
	pub fn new(reader: T, source: String) -> Self {
		Lexer {
			// source,
			reader: BufReader::new(reader),
			curr_char: '\0',
			buffered_char: None,
			curr_location: Location::new_z(0, 0, source),
			line: 0,
			col: 0,
		}
	}

	pub fn next_token(&mut self) -> Result<Token, Error> {
		let mut buff = String::default();

		self.next_char()?;

		while self.curr_char.is_whitespace() {
			self.next_char()?;
		}

		self.curr_location = self.new_location();

		let mut mode: LexerMode =
			if self.curr_char.is_alphabetic() || self.curr_char == '_' || self.curr_char == '@' {
				LexerMode::Word
			} else if self.curr_char.is_numeric() {
				LexerMode::Number
			} else if self.curr_char == '"' {
				LexerMode::String(true, false, false)
			} else if is_operator(self.curr_char) {
				LexerMode::Operator
			} else {
				return Err(Error::InvalidCharacter(
					self.curr_location.clone(),
					self.curr_char,
				));
			};

		let mut no_next_char = true;

		loop {
			let res = match mode {
				LexerMode::Word => self.handle_word(&mut buff),
				LexerMode::Number => self.handle_number(&mut buff),
				LexerMode::String(_, _, _) => self.handle_string(&mut buff, &mut mode),
				LexerMode::Operator => self.handle_operator(&mut buff, &mut mode),
			};

			if let Result::Ok(complete) = res {
				if complete {
					break;
				}
			}

			if let Result::Err(error) = self.next_char() {
				if let Error::EndOfFile = error {
					no_next_char = true;
					break;
				}

				return Err(error);
			}

			no_next_char = false;
		}

		let res = match mode {
			LexerMode::Word => self.finalize_word(&buff),
			LexerMode::Number => self.finalize_number(&buff),
			LexerMode::String(_, _, _) => self.finalize_string(&buff),
			LexerMode::Operator => self.finalize_operator(&buff),
		};

		if !no_next_char {
			self.buffered_char = Some(self.curr_char);
		}

		if let Ok(token) = res {
			if token.content == "#" {
				while self.curr_char != '\n' {
					self.next_char()?;
				}

				return self.next_token();
			}
			Ok(token)
		} else {
			res
		}
	}

	fn handle_word(&mut self, buff: &mut String) -> Result<bool, Error> {
		let c = self.curr_char;

		if !buff.is_empty() && !c.is_alphanumeric() && c != '_' {
			return Ok(true);
		}

		buff.push(self.curr_char);

		Ok(false)
	}

	// TODO: handle different base (ie: other than base 10)
	fn handle_number(&mut self, buff: &mut String) -> Result<bool, Error> {
		let c = self.curr_char;

		if !c.is_numeric() {
			if c.is_alphabetic() || c == '_' {
				return Err(Error::InvalidCharacter(self.new_location(), c));
			}
			return Ok(true);
		}

		buff.push(self.curr_char);

		Ok(false)
	}

	fn handle_string(&mut self, buff: &mut String, mode: &mut LexerMode) -> Result<bool, Error> {
		let c = self.curr_char;

		if let LexerMode::String(first, complete, escape) = mode {
			if *complete {
				Ok(true)
			} else if c != '"' || *escape || *first {
				*first = false;
				*escape = false;

				buff.push(c);

				Ok(false)
			} else if c == '\\' {
				*escape = true;

				buff.push(c);

				Ok(false)
			} else {
				buff.push(c);

				*complete = true;
				Ok(false)
			}
		} else {
			Err(Error::Internal)
		}
	}

	fn handle_operator(&mut self, buff: &mut String, mode: &mut LexerMode) -> Result<bool, Error> {
		let c = self.curr_char;

		if buff.is_empty() {
			buff.push(c);
			return Ok(false);
		}

		if buff.len() > 2 {
			return Ok(true);
		}

		if buff.len() == 2 {
			if buff == ">>" && c == '>' {
				buff.push(c);
				return Ok(false);
			}
			return Ok(true);
		}

		// TODO: handle more radix
		if buff.starts_with('-') && c.is_numeric() {
			buff.push(c);
			*mode = LexerMode::Number;
			return Ok(false);
		}

		if buff.starts_with(c) {
			return match c {
				'-' | '+' | '=' | '/' | '&' | '|' => {
					buff.push(c);
					Ok(false)
				}
				_ => Ok(true),
			};
		}

		if (buff.starts_with('>') || buff.starts_with('<') || buff.starts_with('!')) && c == '=' {
			buff.push(c);
			return Ok(false);
		}

		Ok(true)
	}

	fn finalize_word(&mut self, buff: &str) -> Result<Token, Error> {
		let tk_type = match buff {
			"let" => TokenType::Keyword,
			"fn" => TokenType::Keyword,
			"if" => TokenType::Keyword,
			"true" | "false" => TokenType::BoolLiteral,
			// Not yet ready... SO DON'T YOU DARE USE IT YOU FILTHY MONSTER
			"@include" => TokenType::SpecialInstruction,
			"@exec" => TokenType::SpecialInstruction,
			v if v.starts_with('@') => {
				return Err(Error::UnknownSpecialInstruction(
					self.curr_location.clone(),
					v.into(),
				))
			}
			_ => TokenType::Identifier,
		};

		Ok(Token {
			content: buff.to_owned(),
			token_type: tk_type,
			location: self.curr_location.clone(),
		})
	}

	fn finalize_number(&mut self, buff: &str) -> Result<Token, Error> {
		Ok(Token {
			content: buff.to_owned(),
			token_type: TokenType::IntegerLiteral,
			location: self.curr_location.clone(),
		})
	}

	fn finalize_string(&mut self, buff: &str) -> Result<Token, Error> {
		Ok(Token {
			content: buff.to_owned(),
			token_type: TokenType::StringLiteral,
			location: self.curr_location.clone(),
		})
	}

	fn finalize_operator(&mut self, buff: &str) -> Result<Token, Error> {
		Ok(Token {
			content: buff.to_owned(),
			token_type: TokenType::Operator,
			location: self.curr_location.clone(),
		})
	}

	fn next_char(&mut self) -> Result<(), Error> {
		if let Some(c) = self.buffered_char.take() {
			self.curr_char = c;
			return Ok(());
		}

		self.col += 1;

		let mut buffer = [0; 1];

		let res = self.reader.read(&mut buffer);
		let n = res.unwrap();

		if n == 0 {
			Err(Error::EndOfFile)
		} else {
			let c = buffer[0];

			if (c & 0x80) == 0 {
				self.curr_char = c as char;
			} else {
				let sup_byte_count: u32;

				if (c & 0x20) == 0 {
					sup_byte_count = 1;
				} else if (c & 0x10) == 0 {
					sup_byte_count = 2;
				} else if (c & 0x08) == 0 {
					sup_byte_count = 3;
				} else {
					return Err(Error::Decoder(self.new_location()));
				}

				let mut string_buf = Vec::new();

				string_buf.push(c);

				for _ in 0..sup_byte_count {
					let res = self.reader.read(&mut buffer);
					let n = res.unwrap();

					if n == 0 {
						return Err(Error::UnexpectedEndOfFile(self.new_location()));
					}

					string_buf.push(buffer[0]);
				}

				let Ok(tmp_str) = std::str::from_utf8(&string_buf) else {
					return Err(Error::InvalidCodePoint(self.new_location()));
				};

				self.curr_char = tmp_str.chars().next().unwrap();
			}

			if self.curr_char == '\n' {
				self.line += 1;
				self.col = 0;
			}

			Ok(())
		}
	}

	fn new_location(&self) -> Location {
		Location::new_z(self.line, self.col, self.curr_location.file().to_owned())
	}
}

// TODO: Lexer: probably more operators?
fn is_operator(c: char) -> bool {
	match c {
		'=' | '(' | ')' | ';' | '#' | ',' | '{' | '}' | '[' | ']' | '!' | '>' | '<' | '&' | '|'
		| '+' | '-' | '*' | '/'
			// | '.'
			// | '|' | '&'
			// | '?' | ':'
			// | ';' | '(' | ')' | '[' | ']' | '{' | '}'
			=> true,
		_ => false
	}
}
