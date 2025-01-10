use std::{
	collections::{HashMap, HashSet},
	fmt::Debug,
};

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
	pub func_expr: Box<Expr>,
	pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ArrayExpr {
	pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
	pub left: Box<Expr>,
	pub right: Box<Expr>,
	pub op: BinaryOp,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
	Compare(Comparison),
	Bool(BooleanOperation),
	Numerical(NumericalOperation),
}

#[derive(Debug, Clone)]
pub struct OperatorNotComparator(pub Token);

impl TryFrom<Token> for BinaryOp {
	type Error = OperatorNotComparator;

	fn try_from(s: Token) -> Result<Self, OperatorNotComparator> {
		match s.content.as_str() {
			"==" => Ok(Self::Compare(Comparison::Equal)),
			"!=" => Ok(Self::Compare(Comparison::NotEqual)),
			"<" => Ok(Self::Compare(Comparison::Less)),
			">" => Ok(Self::Compare(Comparison::Greater)),
			"<=" => Ok(Self::Compare(Comparison::LessOrEqual)),
			">=" => Ok(Self::Compare(Comparison::GreaterOrEqual)),
			"||" => Ok(Self::Bool(BooleanOperation::Or)),
			"&&" => Ok(Self::Bool(BooleanOperation::And)),
			"+" => Ok(Self::Numerical(NumericalOperation::Add)),
			"-" => Ok(Self::Numerical(NumericalOperation::Sub)),
			"*" => Ok(Self::Numerical(NumericalOperation::Mul)),
			"/" => Ok(Self::Numerical(NumericalOperation::Div)),
			_ => Err(OperatorNotComparator(s)),
		}
	}
}

#[derive(Debug, Clone)]
pub enum NumericalOperation {
	Add,
	Sub,
	Mul,
	Div,
}

#[derive(Debug, Clone)]
pub enum BooleanOperation {
	Or,
	And,
	// Xor,
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
pub struct VarDecl {
	pub name: String,
	pub val: Expr,
}

#[derive(Debug, Clone)]
pub struct Assign {
	pub target: Expr,
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
pub struct StructDecl {
	pub name: String,
	pub vars: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct StructInstanceExpr {
	pub name: String,
	pub vars_init: HashMap<String, Expr>,
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
	pub source: Box<Expr>,
	pub member_name: String,
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
	Binary(BinaryExpr),
	StructInstance(StructInstanceExpr),
	Member(MemberExpr),
}

impl Expr {
	pub fn is_assignable(&self) -> bool {
		match self {
			Self::Identifier(_) | Self::Member(_) => true,
			_ => false,
		}
	}
}

#[derive(Debug, Clone)]
pub enum ParsedHighLevel {
	Noop,
	VarDecl(VarDecl),
	Assign(Assign),
	FuncDecl(FuncDecl),
	If(IfStatement),
	StructDecl(StructDecl),
	ExprStatement(Expr),
}
