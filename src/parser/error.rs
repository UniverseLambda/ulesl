use std::num::ParseIntError;

use thiserror::Error;

use crate::lexer::{self, Token};

use super::OperatorNotComparator;

pub type Result<T> = std::result::Result<T, ParserError>;

#[derive(Error, Debug)]
pub enum ParserError {
	#[error("{0}")]
	Lexer(#[from] lexer::Error),
	#[error("{}: unexpected token \"{}\", expected: {1:?}", .0.location, .0.content)]
	UnexpectedToken(Token, Option<String>),
	#[error("Invalid number: \"{0}\"")]
	IntegerParsing(String, Option<ParseIntError>),
	#[error("Unexpected End of File")]
	UnexpectedEndOfFile,
}

impl From<(String, ParseIntError)> for ParserError {
	fn from(value: (String, ParseIntError)) -> Self {
		Self::IntegerParsing(value.0, Some(value.1))
	}
}

impl From<OperatorNotComparator> for ParserError {
	fn from(value: OperatorNotComparator) -> Self {
		Self::UnexpectedToken(value.0, Some("==, !=, <, <=, > or >=".to_string()))
	}
}
