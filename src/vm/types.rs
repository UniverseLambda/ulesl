use std::fmt::Display;

use crate::parser::FuncDecl;

use super::FunctionData;

pub trait VmTypable {
	fn get_typeinfo(&self) -> VmType;
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmType {
	Vary,
	Unit,
	Bool,
	Integer,
	String,
	// ReadStream,
	// WriteStream,
	Array,
}

impl Display for VmType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}

impl From<FuncDecl> for (String, FunctionData) {
	fn from(value: FuncDecl) -> Self {
		(
			value.name,
			FunctionData {
				args: value.args,
				packages: value.block.statements,
				// return_type: VmType::Vary,
			},
		)
	}
}
