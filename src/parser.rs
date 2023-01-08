/**
 * This file is part of EasyScriptingLanguage.
 *
 * EasyScriptingLanguage is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * EasyScriptingLanguage is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with EasyScriptingLanguage.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{io::Read, num::ParseIntError};

use crate::lexer::{self, Lexer, Token, TokenType};

#[derive(Debug)]
pub struct FuncCallExpr {
	pub name: String,
	pub args: Vec<Expr>
}

#[derive(Debug)]
pub struct VarAssign {
	pub name: String,
	pub val: Expr,
}

#[derive(Debug)]
pub enum Expr {
	IntLiteral(i64),
	StringLiteral(String),
	Identifier(String),
	FuncCall(FuncCallExpr),
}

#[derive(Debug)]
pub enum ParsedHighLevel {
	VarDecl(VarAssign),
	VarSet(VarAssign),
	FuncCall(FuncCallExpr),
}

#[derive(Debug)]
pub struct ParsedPackage {
	pub source: String,
	pub parsed: ParsedHighLevel,
}

pub struct Parser<T: Read> {
	lexer: Lexer<T>,
	source: String,
	stored_token: Option<Token>,
}

type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
	Lexer(lexer::Error),
	UnexpectedToken{token: Token, source: String},
	IntegerParsing(String, Option<ParseIntError>),
}

impl From<lexer::Error> for ParserError {
    fn from(value: lexer::Error) -> Self {
		Self::Lexer(value)
    }
}

impl From<(String, ParseIntError)> for ParserError {
    fn from(value: (String, ParseIntError)) -> Self {
		Self::IntegerParsing(value.0, Some(value.1))
    }
}

impl<T: Read> Parser<T> {
	pub fn new(lexer: Lexer<T>, source: String) -> Self {
		Parser { lexer, source, stored_token: None }
	}

	pub fn next_package(&mut self) -> Result<Option<ParsedPackage>> {
		let init_token = match self.next_token() {
			Ok(v) => v,
			Err(lexer::Error::EndOfFile) => return Ok(None),
			Err(err) => return Err(err.into())
		};

		match init_token.token_type {
			TokenType::Identifier => (),
			_ => return self.unexpected_token(init_token)
		}

		let high_level = if init_token.content == "var" {
			ParsedHighLevel::VarDecl(self.parse_var_assign(None)?)
		} else {
			let discr_tk = self.next_token()?;

			if discr_tk.token_type != TokenType::Operator {
				return self.unexpected_token(discr_tk);
			}

			let discr_content = discr_tk.content.clone();

			self.store_token(discr_tk);

			if discr_content == "(" {
				ParsedHighLevel::FuncCall(self.parse_func_call(Some(init_token.content))?)
			} else if discr_content == "=" {
				ParsedHighLevel::VarSet(self.parse_var_assign(Some(init_token.content))?)
			} else {
				let tk = self.next_token()?;

				return self.unexpected_token(tk);
			}
		};

		self.expect_end_of_package()?;

		Ok(Some(ParsedPackage { source: self.source.clone(), parsed: high_level }))
	}

	fn parse_var_assign(&mut self, name: Option<String>) -> Result<VarAssign> {
		let name = if let Some(name) = name {
			name
		} else {
			let name_tk = self.next_token()?;

			let TokenType::Identifier = name_tk.token_type else {
				return self.unexpected_token(name_tk);
			};

			if name_tk.content == "var" {
				return self.unexpected_token(name_tk);
			}

			name_tk.content
		};

		let assign_tk = self.next_token()?;

		self.expect_token(&assign_tk, TokenType::Operator, "=")?;

		let val = self.parse_expr()?;

		Ok(VarAssign { name: name, val: val })
	}

	fn parse_func_call(&mut self, func_identifier: Option<String>) -> Result<FuncCallExpr> {
		todo!()
	}

	fn parse_expr(&mut self) -> Result<Expr> {
		let expr_start = self.next_token()?;

		// TODO: extended expressions (func, calculs, etc...)

		Ok(match expr_start.token_type {
			TokenType::IntegerLiteral => Expr::IntLiteral(expr_start.content.parse().map_err(|e| { (expr_start.content.clone(), e) })?),
			TokenType::StringLiteral => Expr::StringLiteral(expr_start.content),
			TokenType::Identifier => return self.parse_branch_identifier_expr(expr_start),
			_ => return self.unexpected_token(expr_start)
		})
	}

	fn parse_branch_identifier_expr(&mut self, identifier: Token) -> Result<Expr> {
		let next_tk = match Self::is_end_of_package(self.next_token())? {
			(true, None) => return Ok(Expr::Identifier(identifier.content)),
			(true, Some(tk))  => { self.store_token(tk); return Ok(Expr::Identifier(identifier.content)) },
			(false, Some(tk)) => tk,
			(false, None) => panic!("Unexpected is_end_of_package result: (false, None)")
		};

		match &next_tk.token_type {
			&TokenType::Operator if next_tk.content == "(" => return Ok(Expr::FuncCall(self.parse_func_call(Some(next_tk.content))?)),
			_ => self.unexpected_token(next_tk),
		}
	}

	fn unexpected_token<R>(&self, tk: Token) -> Result<R> {
		Err(ParserError::UnexpectedToken { token: tk, source: self.source.clone() })
	}

	fn store_token(&mut self, tk: Token) {
		assert!(self.stored_token.is_none());

		self.stored_token = Some(tk);
	}

	fn next_token(&mut self) -> std::result::Result<Token, lexer::Error> {
		if let Some(token) = self.stored_token.take() {
			Ok(token)
		} else {
			self.lexer.next_token()
		}
	}

	fn expect_token(&self, tk: &Token, tk_type: TokenType, content: &str) -> Result<()> {
		self.expect_token_type(tk, tk_type).and(self.expect_token_content(tk, content))
	}

	fn expect_token_content(&self, tk: &Token, content: &str) -> Result<()> {
		if tk.content != content {
			return self.unexpected_token(tk.clone());
		}

		Ok(())
	}

	fn expect_token_type(&self, tk: &Token, tk_type: TokenType) -> Result<()> {
		if tk.token_type != tk_type {
			return self.unexpected_token(tk.clone());
		}

		Ok(())
	}

	fn is_end_of_package(tk_res: std::result::Result<Token, lexer::Error>) -> Result<(bool, Option<Token>)> {
		let tk = match tk_res {
			Ok(v) => v,
			Err(lexer::Error::EndOfFile) => return Ok((true, None)),
			Err(err) => return Err(err.into())
		};

		match tk {
			Token { token_type: TokenType::LineReturn, .. } => Ok((true, Some(tk))),
			Token { token_type: TokenType::Operator, .. } if tk.content == ";" => Ok((true, Some(tk))),
			_ => Ok((false, Some(tk)))
		}
	}

	fn expect_end_of_package(&mut self) -> Result<()> {
		let tk = match self.next_token() {
			Ok(v) => v,
			Err(lexer::Error::EndOfFile) => return Ok(()),
			Err(err) => return Err(err.into())
		};

		match tk {
			Token { token_type: TokenType::LineReturn, .. } => Ok(()),
			Token { token_type: TokenType::Operator, .. } if tk.content == ";" => Ok(()),
			_ => self.unexpected_token(tk)
		}
	}
}
