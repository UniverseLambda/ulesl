use std::{cmp::Ordering, fmt::Debug, str::FromStr};

use crate::{common::Location, lexer::Token};

#[derive(Clone, Debug)]
pub struct LocatedType<T: Clone + Debug> {
	pub inner: T,
	pub location: Location,
}

impl<T: Clone + Debug> LocatedType<T> {
	pub fn new(inner: T, location: Location) -> Self {
		Self { location, inner }
	}
}

impl<T: Clone + Debug> AsRef<T> for LocatedType<T> {
	fn as_ref(&self) -> &T {
		&self.inner
	}
}

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

#[derive(Debug, Clone)]
pub struct CompareExpr {
	pub left: Box<Expr>,
	pub right: Box<Expr>,
	pub comparison: Comparison,
}

#[derive(Debug, Clone)]
pub enum Comparison {
	Less,
	LessOrEqual,
	Equal,
	NotEqual,
	GreaterOrEqual,
	Greater,
}

#[derive(Debug, Clone)]
pub struct OperatorNotComparator(pub Token);

impl TryFrom<Token> for Comparison {
	type Error = OperatorNotComparator;

	fn try_from(s: Token) -> Result<Self, OperatorNotComparator> {
		match s.content.as_str() {
			"==" => Ok(Self::Equal),
			"!=" => Ok(Self::NotEqual),
			"<" => Ok(Self::Less),
			">" => Ok(Self::Greater),
			"<=" => Ok(Self::LessOrEqual),
			">=" => Ok(Self::GreaterOrEqual),
			_ => Err(OperatorNotComparator(s)),
		}
	}
}

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
	pub statements: Vec<LocatedType<ParsedHighLevel>>,
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
	Compare(CompareExpr),
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
