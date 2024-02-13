use thiserror::Error;

use std::{io::Read, num::ParseIntError};

use crate::lexer::{self, Lexer, Token, TokenType};

#[derive(Debug, Clone)]
pub struct IfStatement {
	pub val: Expr,
	pub block: StatementBlock,
}

#[derive(Debug, Clone)]
pub struct FuncCallExpr {
	pub name: String,
	pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ArrayExpr {
	pub args: Vec<Expr>,
}

// #[derive(Debug, Clone)]
// pub struct VarDecl {
// 	pub assign: VarAssign,
// }

// impl Deref for VarDecl {
// 	type Target = VarAssign;

// 	fn deref(&self) -> &Self::Target {
// 		&self.assign
// 	}
// }

// impl DerefMut for VarDecl {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		&mut self.assign
// 	}
// }

#[derive(Debug, Clone)]
pub struct VarAssign {
	pub name: String,
	pub val: Expr,
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
	pub name: String,
	pub args: Vec<String>,
	// pub ret_type: VmType,
	pub block: StatementBlock,
}

#[derive(Debug, Clone)]
pub struct StatementBlock {
	pub statements: Vec<ParsedPackage>,
	// pub ret_type: VmType,
}

#[derive(Debug, Clone)]
pub enum Expr {
	IntLiteral(i64),
	StringLiteral(String),
	BoolLiteral(bool),
	Identifier(String),
	FuncCall(FuncCallExpr),
	Array(ArrayExpr),
}

#[derive(Debug, Clone)]
pub enum ParsedHighLevel {
	Noop,
	VarDecl(VarAssign),
	VarSet(VarAssign),
	FuncDecl(FuncDecl),
	FuncCall(FuncCallExpr),
	If(IfStatement),
}

#[derive(Debug, Clone)]
pub struct ParsedPackage {
	pub source: String,
	pub parsed: ParsedHighLevel,
}

pub struct Parser<T: Read> {
	lexer: Lexer<T>,
	source: String,
	stored_token: Option<Token>,
}

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

impl<T: Read> Parser<T> {
	pub fn new(lexer: Lexer<T>, source: String) -> Self {
		Parser {
			lexer,
			source,
			stored_token: None,
		}
	}

	pub fn next_package(&mut self) -> Result<Option<ParsedPackage>> {
		let init_token = {
			match self.next_token() {
				Ok(v) => v,
				Err(lexer::Error::EndOfFile) => return Ok(None),
				Err(err) => return Err(err.into()),
			}
		};

		let tktype_ident = self.expect_token_type(&init_token, TokenType::Identifier);
		let tktype_keyword = self.expect_token_type(&init_token, TokenType::Keyword);

		tktype_ident.or(tktype_keyword)?;

		let high_level = if let TokenType::Keyword = init_token.token_type {
			match init_token.content.as_str() {
				"let" => ParsedHighLevel::VarDecl(self.parse_var_assign(None)?),
				"fn" => ParsedHighLevel::FuncDecl(self.parse_func_decl()?),
				"if" => ParsedHighLevel::If(self.parse_if_statement()?),
				_ => return self.unexpected_token(init_token),
			}
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

		Ok(Some(ParsedPackage {
			source: self.source.clone(),
			parsed: high_level,
		}))
	}

	fn parse_var_assign(&mut self, name: Option<String>) -> Result<VarAssign> {
		let name = if let Some(name) = name {
			name
		} else {
			let name_tk = self.next_token()?;

			self.expect_token_type(&name_tk, TokenType::Identifier)?;

			name_tk.content
		};

		let assign_tk = self.next_token()?;

		self.expect_token(&assign_tk, TokenType::Operator, "=")?;

		let val = self.parse_expr()?;

		Ok(VarAssign { name, val })
	}

	fn parse_func_decl(&mut self) -> Result<FuncDecl> {
		let func_ident = {
			let func_identifier_tk = self.next_token()?;

			self.expect_token_type(&func_identifier_tk, TokenType::Identifier)?;

			func_identifier_tk.content
		};

		{
			let tk = self.next_token()?;
			self.expect_token_content(&tk, "(")?;
		}

		let mut args: Vec<String> = Vec::new();

		let mut first_arg_token: bool = true;
		let mut next_token;

		loop {
			next_token = self.next_token()?;

			if next_token.content == ")" {
				break;
			}

			if !first_arg_token {
				self.expect_token_content(&next_token, ",")?;
				first_arg_token = false;

				next_token = self.next_token()?;
			}

			self.expect_token_type(&next_token, TokenType::Identifier)?;

			args.push(next_token.content);
		}

		let block = self.parse_statement_block()?;

		Ok(FuncDecl {
			name: func_ident,
			args,
			block,
		})
	}

	fn parse_if_statement(&mut self) -> Result<IfStatement> {
		let val = self.parse_expr()?;
		let block = self.parse_statement_block()?;

		Ok(IfStatement { val, block })
	}

	fn parse_statement_block(&mut self) -> Result<StatementBlock> {
		{
			let tk = self.next_token()?;
			self.expect_token_content(&tk, "{")?;
		}

		let mut statements = Vec::new();

		loop {
			let next_tk = self.next_token()?;

			if next_tk.content == "}" {
				break;
			}

			self.store_token(next_tk);

			let Some(pkg) = self.next_package()? else {
				return Err(ParserError::UnexpectedEndOfFile);
			};

			statements.push(pkg);
		}

		Ok(StatementBlock { statements })
	}

	fn parse_func_call(&mut self, func_identifier: Option<String>) -> Result<FuncCallExpr> {
		let func_identifier = if let Some(func_identifier) = func_identifier {
			func_identifier
		} else {
			let func_identifier_tk = self.next_token()?;

			self.expect_token_type(&func_identifier_tk, TokenType::Identifier)?;

			func_identifier_tk.content
		};

		let parenthese_tk = self.next_token()?;

		self.expect_token(&parenthese_tk, TokenType::Operator, "(")?;

		Ok(FuncCallExpr {
			name: func_identifier,
			args: self.parse_list(")")?,
		})
	}

	fn parse_expr(&mut self) -> Result<Expr> {
		let expr_start = self.next_token()?;

		// TODO: extended expressions (calculs, etc...)

		Ok(match expr_start.token_type {
			TokenType::IntegerLiteral => Expr::IntLiteral(
				expr_start
					.content
					.parse()
					.map_err(|e| (expr_start.content.clone(), e))?,
			),
			TokenType::StringLiteral => Expr::StringLiteral(expr_start.content),
			// UNWRAP: BoolLiteral has already been checked
			TokenType::BoolLiteral => Expr::BoolLiteral(expr_start.content.parse().unwrap()),
			TokenType::Identifier => return self.parse_branch_identifier_expr(expr_start),
			TokenType::Operator if expr_start.content == "[" => return self.parse_array(),
			_ => return self.unexpected_token(expr_start),
		})
	}

	fn parse_branch_identifier_expr(&mut self, identifier: Token) -> Result<Expr> {
		let next_tk = match Self::is_end_of_package(self.next_token())? {
			(true, None) => return Ok(Expr::Identifier(identifier.content)),
			(true, Some(tk)) => {
				self.store_token(tk);
				return Ok(Expr::Identifier(identifier.content));
			}
			(false, Some(tk)) => tk,
			(false, None) => panic!("Unexpected is_end_of_package result: (false, None)"),
		};

		match next_tk.token_type {
			TokenType::Operator if next_tk.content == "(" => {
				Ok(Expr::FuncCall(self.parse_func_call(Some(next_tk.content))?))
			}
			TokenType::Operator
				if next_tk.content == ")" || next_tk.content == "," || next_tk.content == "]" =>
			{
				self.store_token(next_tk);

				Ok(Expr::Identifier(identifier.content))
			}
			_ => self.unexpected_token(next_tk),
		}
	}

	fn parse_array(&mut self) -> Result<Expr> {
		Ok(Expr::Array(ArrayExpr {
			args: self.parse_list("]")?,
		}))
	}

	fn parse_list(&mut self, stop_op: &str) -> Result<Vec<Expr>> {
		let mut args = Vec::new();

		let mut first = true;

		loop {
			let mut tk = self.next_token()?;

			if tk.token_type == TokenType::Operator && tk.content == stop_op {
				break;
			} else {
				if !first {
					self.expect_token(&tk, TokenType::Operator, ",")?;

					tk = self.next_token()?;
				} else {
					first = false;
				}

				self.store_token(tk);

				args.push(self.parse_expr()?);
			}
		}

		Ok(args)
	}

	fn unexpected_token<R>(&self, tk: Token) -> Result<R> {
		Err(ParserError::UnexpectedToken(tk, None))
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
		self.expect_token_type(tk, tk_type)
			.and(self.expect_token_content(tk, content))
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

	fn is_end_of_package(
		tk_res: std::result::Result<Token, lexer::Error>,
	) -> Result<(bool, Option<Token>)> {
		let tk = match tk_res {
			Ok(v) => v,
			Err(lexer::Error::EndOfFile) => return Ok((true, None)),
			Err(err) => return Err(err.into()),
		};

		match tk {
			Token {
				token_type: TokenType::Operator,
				..
			} if tk.content == ";" => Ok((true, Some(tk))),
			_ => Ok((false, Some(tk))),
		}
	}

	fn expect_end_of_package(&mut self) -> Result<()> {
		let tk = match self.next_token() {
			Ok(v) => v,
			Err(lexer::Error::EndOfFile) => return Ok(()),
			Err(err) => return Err(err.into()),
		};

		match tk {
			Token {
				token_type: TokenType::Operator,
				..
			} if tk.content == ";" => Ok(()),
			_ => self.unexpected_token(tk),
		}
	}
}
