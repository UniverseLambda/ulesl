use std::fmt::Display;

use thiserror::Error;

use crate::common::Location;

pub type VmResult<T> = std::result::Result<T, VmError>;

#[derive(Debug)]
pub struct VmError {
	err_type: VmErrorType,
	context: Box<VmErrorContext>,
}

impl Display for VmError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.context.as_ref() {
			VmErrorContext::Internal => "vm_internals".fmt(f)?,
			/* VmErrorContext::Location { location } |*/
			VmErrorContext::FuncCall { location, .. } => {
				location.file().fmt(f)?;
				':'.fmt(f)?;
				location.line().fmt(f)?;
				location.column().fmt(f)?;
			}
		}

		": ".fmt(f)?;

		if let VmErrorContext::FuncCall {
			func_name,
			arg_name,
			..
		} = self.context.as_ref()
		{
			"in ".fmt(f)?;
			func_name.fmt(f)?;

			if let Some(arg_name) = arg_name {
				", at argument ".fmt(f)?;
				arg_name.fmt(f)?;
			}

			": ".fmt(f)?;
		}

		self.err_type.fmt(f)
	}
}

impl VmError {
	pub fn new(err_type: VmErrorType) -> Self {
		Self {
			err_type,
			context: Box::default(),
		}
	}

	// pub fn with_context_internal(mut self) -> Self {
	// 	self.context = Box::from(VmErrorContext::Internal);

	// 	self
	// }

	// pub fn with_context_location(mut self, location: Location) -> Self {
	// 	self.context = Box::from(VmErrorContext::Location { location });

	// 	self
	// }

	pub fn with_context_func_call(mut self, location: Location, func_name: String) -> Self {
		self.context = Box::new(VmErrorContext::FuncCall {
			location,
			func_name,
			arg_name: None,
		});

		self
	}

	pub fn with_context_func_arg(
		mut self,
		location: Location,
		func_name: String,
		arg_name: String,
	) -> Self {
		self.context = Box::new(VmErrorContext::FuncCall {
			location,
			func_name,
			arg_name: Some(arg_name),
		});

		self
	}

	pub fn unknown_identifier(name: String) -> Self {
		Self {
			err_type: VmErrorType::UnknownIdentifier(name),
			context: Box::default(),
		}
	}

	pub fn func_name_dup(name: String) -> Self {
		Self {
			err_type: VmErrorType::FuncNameDuplicate(name),
			context: Box::default(),
		}
	}

	pub fn var_name_dup(name: String) -> Self {
		Self {
			err_type: VmErrorType::VarNameDuplicate(name),
			context: Box::default(),
		}
	}

	pub fn wrong_arg_count(limit: usize, got: usize) -> Self {
		let err_type = if got < limit {
			VmErrorType::NotEnoughArg {
				expected: limit,
				got,
			}
		} else {
			VmErrorType::TooMuchArgs {
				expected: limit,
				got,
			}
		};

		Self {
			err_type,
			context: Box::default(),
		}
	}

	pub fn invalid_value_type(expected: String, got: String) -> Self {
		Self {
			err_type: VmErrorType::InvalidValueType { expected, got },
			context: Box::default(),
		}
	}

	// pub fn invalid_string(raw_string: String, invalid_char_idx: usize) -> Self {
	// 	Self {
	// 		err_type: VmErrorType::InvalidString {
	// 			raw_string,
	// 			invalid_char_idx,
	// 		},
	// 		context: Box::default(),
	// 	}
	// }

	pub fn invalid_escape(raw_string: String, invalid_escape_idx: usize) -> Self {
		Self {
			err_type: VmErrorType::InvalidEscape {
				raw_string,
				invalid_escape_idx,
			},
			context: Box::default(),
		}
	}
}

#[derive(Debug, Default)]
pub enum VmErrorContext {
	/* No context, error generated by VM internals */
	#[default]
	Internal,
	/* Minimal context, we just know it was emitted when evaluating a package */
	// Location {
	// 	location: Location,
	// },
	/* Full known context, we know the error was emitted when trying to call a function */
	FuncCall {
		location: Location,
		func_name: String,
		arg_name: Option<String>,
	},
}

#[derive(Debug, Error)]
pub enum VmErrorType {
	#[error("unknown identifier: {0}")]
	UnknownIdentifier(String),
	#[error("duplicate function: {0}")]
	FuncNameDuplicate(String),
	#[error("duplicate variable: {0}")]
	VarNameDuplicate(String),
	#[error("not enough argument (expected {expected}, got {got})")]
	NotEnoughArg { expected: usize, got: usize },
	#[error("too many arguments (expected {expected}, got {got})")]
	TooMuchArgs { expected: usize, got: usize },
	#[error("unexpected type (expected {expected}, got {got})")]
	InvalidValueType { expected: String, got: String },
	// #[error("invalid string value: invalid char at {invalid_char_idx}")]
	// InvalidString {
	// 	raw_string: String,
	// 	invalid_char_idx: usize,
	// },
	#[error("invalid string value: invalid escape sequence at {invalid_escape_idx}")]
	InvalidEscape {
		raw_string: String,
		invalid_escape_idx: usize,
	},
	#[error("invalid comparison: could not compare types {left_type} and {right_type}")]
	InvalidComparison {
		left_type: String,
		right_type: String,
	},
}

pub trait VmResultExt {
	// fn with_context_internal(self) -> Self;
	// fn with_context_location(self, location: Location) -> Self;
	fn with_context_func_call(self, location: Location, func_name: String) -> Self;
	fn with_context_func_arg(self, location: Location, func_name: String, arg_name: String)
		-> Self;
}

impl<T> VmResultExt for VmResult<T> {
	// fn with_context_internal(self) -> Self {
	// 	self.map_err(VmError::with_context_internal)
	// }

	// fn with_context_location(self, location: Location) -> Self {
	// 	self.map_err(|v| v.with_context_location(location))
	// }

	fn with_context_func_call(self, location: Location, func_name: String) -> Self {
		self.map_err(|v| v.with_context_func_call(location, func_name))
	}

	fn with_context_func_arg(
		self,
		location: Location,
		func_name: String,
		arg_name: String,
	) -> Self {
		self.map_err(|v| v.with_context_func_arg(location, func_name, arg_name))
	}
}
