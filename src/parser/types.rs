use std::fmt::Debug;

use crate::common::Location;

#[derive(Debug)]
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
