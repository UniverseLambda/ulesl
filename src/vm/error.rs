pub type Result<T> = std::result::Result<T, VmError>;

#[derive(Debug)]
pub enum VmError {
	FuncNameDuplicate(String),
	FuncNameNotFound(String),
	VarNameDuplicate(String),
	VarNameNotFound(String),
}
