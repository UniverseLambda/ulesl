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

impl From<FuncDecl> for (String, FunctionData) {
	fn from(value: FuncDecl) -> Self {
		(
			value.name,
			FunctionData {
				args: value.args,
				packages: value.block.statements,
				return_type: VmType::Vary,
			},
		)
	}
}
