use std::{fmt::{Display, Write}, rc::Rc};

use crate::parser;

use super::types::{VmType, VmTypable};

#[derive(Clone, Debug)]
pub enum VmVariant {
	Unit,
	Bool(bool),
	Integer(i64),
	String(String),
	// ReadStream(Box<dyn Read>),
	// WriteStream(Box<dyn Write>),
	Array(Vec<VmVariant>),
	Ref(Rc<VmVariant>),
}

impl VmTypable for VmVariant {
	fn get_typeinfo(&self) -> VmType {
		match self {
			VmVariant::Unit			=> VmType::Unit,
			VmVariant::Bool(_)		=> VmType::Bool,
			VmVariant::Integer(_)	=> VmType::Integer,
			VmVariant::String(_)	=> VmType::String,
			VmVariant::Array(_)		=> VmType::Array,
			VmVariant::Ref(_)		=> todo!()
		}
	}
}

impl From<parser::Expr> for VmVariant {
	fn from(value: parser::Expr) -> Self {
		match value {
			parser::Expr::IntLiteral(v) => Self::Integer(v),
			parser::Expr::StringLiteral(v) => Self::String(v),
			parser::Expr::Identifier(_) => unimplemented!(),
			parser::Expr::FuncCall(_) => unimplemented!(),
		}
	}
}

impl Display for VmVariant {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			VmVariant::Unit => f.write_str("()"),
			VmVariant::Bool(v) => v.fmt(f),
			VmVariant::Integer(v) => v.fmt(f),
			VmVariant::String(v) => v.fmt(f),
			VmVariant::Array(array) => {
				f.write_char('[')?;

				for elem in array {
					elem.fmt(f)?;
				}

				f.write_char(']')
			},
			VmVariant::Ref(v) => v.fmt(f)
		}
	}
}
