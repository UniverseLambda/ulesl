pub type Result<T> = std::result::Result<T, VmError>;

#[derive(Debug)]
pub enum VmError {
	FuncNameDuplicate(String),
	FuncNameNotFound(String),
	VarNameDuplicate(String),
	VarNameNotFound(String),
	NotEnoughArg { func_name: String, expected: usize, got: usize },
	TooMuchArgs { func_name: String, expected: usize, got: usize },
	InvalidArgType { func_name: String, arg_name: String, expected: String, got: String },
	InvalidString { raw_string: String, invalid_char_idx: usize },
	InvalidEscape { raw_string: String, invalid_escape_idx: usize },
}
