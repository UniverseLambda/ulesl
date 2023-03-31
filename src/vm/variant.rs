use std::{fmt::{Display, Write}, rc::Rc};

use crate::parser;

use super::{types::{VmType, VmTypable}, error::{Result, VmError}};

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

impl VmVariant {
	pub fn new_from_string_expr(str: &str) -> Result<Self> {
		let trimmed_str = &str[1..(str.len() - 1)];
		let mut res_str = String::with_capacity(trimmed_str.len());

		let mut escaped = false;

		for (idx, c) in trimmed_str.chars().enumerate() {
			if escaped {
				escaped = false;

				match c {
					'n' 	=> res_str.push('\n'),
					'r' 	=> res_str.push('\r'),
					't' 	=> res_str.push('\t'),
					'\\' 	=> res_str.push('\\'),
					'0' 	=> res_str.push('\t'),
					'\'' 	=> res_str.push('\''),
					'\"' 	=> res_str.push('\"'),
					_ 		=> return Err(VmError::InvalidEscape { raw_string: str.to_owned(), invalid_escape_idx: idx - 1 })
				}

				continue;
			}

			if c == '\\' {
				escaped = true;
				continue;
			}

			res_str.push(c);
		}

		Ok(Self::String(res_str))
	}

	pub fn unwrap_unit(self) -> () {
		if let VmVariant::Unit = self {
			return ()
		}

		panic!("Expected VM variant Unit, got {:?}", self.get_typeinfo());
	}

	pub fn unwrap_bool(self) -> bool {
		if let VmVariant::Bool(b) = self {
			return b;
		}

		panic!("Expected VM variant Bool, got {:?}", self.get_typeinfo());
	}

	pub fn unwrap_integer(self) -> i64 {
		if let VmVariant::Integer(v) = self {
			return v;
		}

		panic!("Expected VM variant Integer, got {:?}", self.get_typeinfo());
	}

	pub fn unwrap_string(self) -> String {
		if let VmVariant::String(v) = self {
			return v;
		}

		panic!("Expected VM variant String, got {:?}", self.get_typeinfo());
	}

	pub fn unwrap_array(self) -> Vec<VmVariant> {
		if let VmVariant::Array(v) = self {
			return v;
		}

		panic!("Expected VM variant Array, got {:?}", self.get_typeinfo());
	}

	pub fn unwrap_ref(self) -> Rc<VmVariant> {
		if let VmVariant::Ref(v) = self {
			return v;
		}

		panic!("Expected VM variant Ref, got {:?}", self.get_typeinfo());
	}
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
			parser::Expr::Array(_) => unimplemented!(),
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

				let mut first = true;

				for elem in array {
					if first {
						first = false;
					} else {
						f.write_str(", ")?;
					}

					elem.fmt(f)?;
				}

				f.write_char(']')
			},
			VmVariant::Ref(v) => v.fmt(f)
		}
	}
}
