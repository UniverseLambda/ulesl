use std::io::Read;

pub mod legacy;

pub use legacy::{
	ArrayExpr, Expr, FuncCallExpr, FuncDecl, IfStatement, ParsedHighLevel, ParsedPackage,
	ParserError, Result, StatementBlock, VarAssign,
};

use crate::lexer::{self, Lexer, Token, TokenType};

pub struct Parser<T: Read> {
	lexer: Lexer<T>,
	source: String,
	current_token: Option<Token>,
	lookahead_token: Option<Token>,
	retain_last_token: bool,
}

impl<T: Read> Parser<T> {
	pub fn new(lexer: Lexer<T>, source: String) -> Self {
		Self {
			lexer,
			source,
			current_token: None,
			lookahead_token: None,
			retain_last_token: false,
		}
	}

	pub fn next_package(&mut self) -> Result<Option<ParsedPackage>> {
		let peeked_token = self.peek_token()?;

		let Some(token) = peeked_token else {
			return Ok(None);
		};

		self.expect_token_type(&token, TokenType::Identifier)
			.or_else(|_| self.expect_token_type(&token, TokenType::Keyword))
			.or_else(|_| self.expect_token(&token, TokenType::Operator, ";"))?;

		let high_level = if let TokenType::Keyword = &token.token_type {
			match token.content.as_str() {
				"let" => ParsedHighLevel::VarDecl(self.parse_var_decl_or_assign()?),
				"fn" => ParsedHighLevel::FuncDecl(self.parse_func_decl()?),
				"if" => ParsedHighLevel::If(self.parse_if_statement()?),
				_ => {
					return self.unexpected_token(
						self.current_token.clone().unwrap(),
						Some("statement or function declaration".to_string()),
					)
				}
			}
		} else if let TokenType::Operator = &token.token_type {
			self.advance_token()?;

			ParsedHighLevel::Noop
		} else {
			self.advance_token()?;

			let disc_tk = self.peek_or_fail()?;

			self.expect_token_type(&disc_tk, TokenType::Operator)?;

			match disc_tk.content.as_str() {
				"(" => {
					self.advance_token()?;

					let args = self.parse_expr_list(")")?;

					let end_tk = self.next_or_fail()?;
					self.expect_token(&end_tk, TokenType::Operator, ";")?;

					ParsedHighLevel::FuncCall(FuncCallExpr {
						name: token.content,
						args,
					})
				}
				"=" => {
					self.retain_token();
					ParsedHighLevel::VarSet(self.parse_var_decl_or_assign()?)
				}
				_ => self.unexpected_token(disc_tk, Some("( or =".to_string()))?,
			}
		};

		Ok(Some(ParsedPackage {
			source: self.source.clone(),
			parsed: high_level,
		}))
	}

	fn parse_var_decl_or_assign(&mut self) -> Result<VarAssign> {
		let next_tk = self.next_or_fail()?;

		let name_tk = if next_tk.content == "let" {
			self.next_or_fail()?
		} else {
			next_tk
		};

		self.expect_token_type(&name_tk, TokenType::Identifier)?;

		let assign_tk = self.next_or_fail()?;

		self.expect_token(&assign_tk, TokenType::Operator, "=")?;

		let val = self.parse_expr()?;

		let end_tk = self.next_or_fail()?;
		self.expect_token(&end_tk, TokenType::Operator, ";")?;

		Ok(VarAssign {
			name: name_tk.content,
			val,
		})
	}

	fn parse_func_decl(&mut self) -> Result<FuncDecl> {
		let fn_keyword = self.next_or_fail()?;
		self.expect_token(&fn_keyword, TokenType::Keyword, "fn")?;

		let func_identifier = self.next_or_fail()?;

		self.expect_token_type(&func_identifier, TokenType::Identifier)?;

		let parenth = self.next_or_fail()?;

		self.expect_token(&parenth, TokenType::Operator, "(")?;

		let arg_list = self.parse_arg_list(")")?;
		let block = self.parse_block()?;

		Ok(FuncDecl {
			name: func_identifier.content,
			args: arg_list,
			block,
		})
	}

	fn parse_if_statement(&mut self) -> Result<IfStatement> {
		let if_statement = self.next_or_fail()?;

		self.expect_token(&if_statement, TokenType::Keyword, "if")?;

		let if_cond = self.parse_expr()?;

		Ok(IfStatement {
			val: if_cond,
			block: self.parse_block()?,
		})
	}

	fn parse_expr(&mut self) -> Result<Expr> {
		// TODO: extended expressions (calculs, etc...)

		if let TokenType::Identifier = self.peek_or_fail()?.token_type {
			return self.parse_branch_identifier_expr();
		}

		// We consume the token as we are the one doing the parsing
		let expr_start = self.next_or_fail()?;

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
			TokenType::Operator if expr_start.content == "[" => Expr::Array(self.parse_array()?),
			_ => return self.unexpected_token(expr_start, Some("expression".to_string())),
		})
	}

	fn parse_branch_identifier_expr(&mut self) -> Result<Expr> {
		let identifier = self.next_or_fail()?;

		let peeked = self.peek_or_fail()?;

		// TODO: implement array access

		if peeked.content == "(" {
			self.advance_token()?;

			let args = self.parse_expr_list(")")?;

			Ok(Expr::FuncCall(FuncCallExpr {
				name: identifier.content,
				args,
			}))
		} else {
			Ok(Expr::Identifier(identifier.content))
		}
	}

	fn parse_array(&mut self) -> Result<ArrayExpr> {
		let expr_list = self.parse_expr_list("]")?;

		Ok(ArrayExpr { args: expr_list })
	}

	fn parse_expr_list(&mut self, end_operator: &str) -> Result<Vec<Expr>> {
		let mut exprs: Vec<Expr> = Vec::new();

		loop {
			let next_token = self.peek_or_fail()?;

			if TokenType::Operator == next_token.token_type && next_token.content == end_operator {
				self.advance_token()?;
				break;
			}

			exprs.push(self.parse_expr()?);

			let end_token = self.next_or_fail()?;

			if TokenType::Operator == end_token.token_type && end_token.content == end_operator {
				break;
			}

			self.expect_token(&end_token, TokenType::Operator, ",")?;
		}

		Ok(exprs)
	}

	fn parse_arg_list(&mut self, end_operator: &str) -> Result<Vec<String>> {
		let mut idents: Vec<String> = Vec::new();

		loop {
			let next_token = self.next_or_fail()?;

			if TokenType::Operator == next_token.token_type && next_token.content == end_operator {
				break;
			}

			self.expect_token_type(&next_token, TokenType::Identifier)?;

			idents.push(next_token.content);

			let end_token = self.next_or_fail()?;

			if TokenType::Operator == end_token.token_type && end_token.content == end_operator {
				break;
			}

			self.expect_token(&end_token, TokenType::Operator, ",")?;
		}

		Ok(idents)
	}

	fn parse_block(&mut self) -> Result<StatementBlock> {
		let block_start = self.next_or_fail()?;

		self.expect_token(&block_start, TokenType::Operator, "{")?;

		let mut statements = Vec::new();

		loop {
			let next_token = self.peek_or_fail()?;

			if TokenType::Operator == next_token.token_type && next_token.content == "}" {
				break;
			}

			let Some(statement) = self.next_package()? else {
				return Err(ParserError::UnexpectedEndOfFile);
			};

			statements.push(statement);
		}

		self.advance_token()?;

		Ok(StatementBlock { statements })
	}

	#[track_caller]
	fn expect_token_type(&self, tk: &Token, tk_type: TokenType) -> Result<()> {
		// println!(
		// 	"expect_token_type called: {}",
		// 	std::panic::Location::caller()
		// );

		if tk.token_type != tk_type {
			return self.unexpected_token(tk.clone(), Some(format!("{tk_type:?}")));
		}

		Ok(())
	}

	#[track_caller]
	fn expect_token(&self, tk: &Token, tk_type: TokenType, content: &str) -> Result<()> {
		// println!("expect_token called: {}", std::panic::Location::caller());

		self.expect_token_type(tk, tk_type)
			.and(self.expect_token_content(tk, content))
	}

	#[track_caller]
	fn expect_token_content(&self, tk: &Token, content: &str) -> Result<()> {
		// println!(
		// 	"expect_token_content called: {}",
		// 	std::panic::Location::caller()
		// );

		if tk.content != content {
			return self.unexpected_token(tk.clone(), Some(content.to_string()));
		}

		Ok(())
	}

	#[track_caller]
	fn unexpected_token<R>(&self, tk: Token, expected: Option<String>) -> Result<R> {
		// println!(
		// 	"unexpected_token called: {}",
		// 	std::panic::Location::caller()
		// );

		Err(ParserError::UnexpectedToken(tk, expected))
	}

	fn peek_or_fail(&mut self) -> Result<Token> {
		let Some(token) = self.peek_token()? else {
			return Err(ParserError::UnexpectedEndOfFile);
		};

		Ok(token)
	}

	fn next_or_fail(&mut self) -> Result<Token> {
		self.advance_token()?;

		let Some(token) = self.current_token.clone() else {
			return Err(ParserError::UnexpectedEndOfFile);
		};

		Ok(token)
	}

	fn peek_token(&mut self) -> Result<Option<Token>> {
		if let Some(token) = self.lookahead_token.clone() {
			return Ok(Some(token));
		}

		self.lookahead_token = self.read_token()?;

		Ok(self.lookahead_token.clone())
	}

	fn retain_token(&mut self) {
		self.retain_last_token = true;
	}

	fn advance_token(&mut self) -> Result<()> {
		if self.retain_last_token {
			self.retain_last_token = false;
			return Ok(());
		}

		if let Some(token) = self.lookahead_token.take() {
			self.current_token = Some(token)
		} else {
			self.current_token = self.read_token()?
		}

		Ok(())
	}

	fn read_token(&mut self) -> Result<Option<Token>> {
		let result = self.lexer.next_token();

		if let Err(lexer::Error::EndOfFile) = result {
			Ok(None)
		} else {
			Ok(Some(result?))
		}
	}
}
