use std::{
	cmp::Ordering,
	collections::HashMap,
	fmt::{Display, Write},
	sync::Arc,
};

use parking_lot::Mutex;

use crate::parser;

use super::{
	error::{VmError, VmResult},
	types::{VmTypable, VmType},
};

#[derive(Clone, Debug)]
pub enum VmVariant {
	Unit,
	Bool(bool),
	Integer(i64),
	String(String),
	Array(Vec<VmVariant>),
	Struct(
		HashMap<String, StoredValue>, /* TODO: Check if having an UID for structs to differentiate them at runtime is a good idea */
	),
	Ref(StoredValue),
}

impl<T: IntoVariant> From<T> for VmVariant {
	fn from(value: T) -> Self {
		value.into_variant()
	}
}

impl VmVariant {
	pub const TRUE: Self = Self::Bool(true);
	pub const FALSE: Self = Self::Bool(false);

	pub fn new_from_string_expr(str: &str) -> VmResult<Self> {
		let trimmed_str = &str[1..(str.len() - 1)];
		let mut res_str = String::with_capacity(trimmed_str.len());

		let mut escaped = false;

		for (idx, c) in trimmed_str.chars().enumerate() {
			if escaped {
				escaped = false;

				match c {
					'n' => res_str.push('\n'),
					'r' => res_str.push('\r'),
					't' => res_str.push('\t'),
					'\\' => res_str.push('\\'),
					'0' => res_str.push('\t'),
					'\'' => res_str.push('\''),
					'\"' => res_str.push('\"'),
					_ => return Err(VmError::invalid_escape(str.to_owned(), idx - 1)),
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

	#[inline]
	pub fn try_native<T: TryFromVariant>(self) -> VmResult<T> {
		T::try_from_variant(self)
	}

	#[inline]
	pub fn compare(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(Self::Unit, Self::Unit) => Some(Ordering::Equal),
			(Self::Bool(a), Self::Bool(b)) => Some(a.cmp(b)),
			(Self::Integer(a), Self::Integer(b)) => Some(a.cmp(b)),
			(Self::String(a), Self::String(b)) => Some(a.cmp(b)),
			(Self::Array(a), Self::Array(b)) => a.partial_cmp(b),
			(Self::Ref(a), Self::Ref(b)) => a.value().compare(&b.value()),
			(Self::Ref(a), b) => a.value().compare(b),
			(a, Self::Ref(b)) => a.compare(&b.value()),
			_ => None,
		}
	}

	#[inline]
	pub fn consume_reference(self) -> Self {
		match self {
			VmVariant::Ref(stored_value) => stored_value.value(),
			v => v,
		}
	}

	#[inline]
	pub fn clone_deref(&self) -> Self {
		match self {
			VmVariant::Ref(stored_value) => stored_value.value(),
			v => v.clone(),
		}
	}
}

impl PartialEq for VmVariant {
	fn eq(&self, other: &Self) -> bool {
		self.compare(other).map_or(false, |v| v == Ordering::Equal)
	}
}

impl PartialOrd for VmVariant {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.compare(other)
	}
}

impl VmTypable for VmVariant {
	fn get_typeinfo(&self) -> VmType {
		match self {
			VmVariant::Unit => VmType::Unit,
			VmVariant::Bool(_) => VmType::Bool,
			VmVariant::Integer(_) => VmType::Integer,
			VmVariant::String(_) => VmType::String,
			VmVariant::Array(_) => VmType::Array,
			VmVariant::Struct(_) => VmType::Struct,
			VmVariant::Ref(sub) => sub.get_typeinfo(),
		}
	}
}

impl From<parser::types::Expr> for VmVariant {
	fn from(value: parser::types::Expr) -> Self {
		match value {
			parser::types::Expr::IntLiteral(v) => Self::Integer(v),
			parser::types::Expr::StringLiteral(v) => Self::String(v),
			parser::types::Expr::BoolLiteral(v) => Self::Bool(v),
			parser::types::Expr::StructInstance(_) => unimplemented!(),
			parser::types::Expr::Array(_) => unimplemented!(),
			parser::types::Expr::Identifier(_) => unimplemented!(),
			parser::types::Expr::FuncCall(_) => unimplemented!(),
			parser::types::Expr::Binary(_) => unimplemented!(),
			parser::types::Expr::Member(_) => unimplemented!(),
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
			}
			VmVariant::Struct(members) => {
				f.write_str("{ ")?;

				let mut first = true;

				for (member_name, member_value) in members {
					if first {
						first = false;
					} else {
						f.write_str(", ")?;
					}

					member_name.fmt(f)?;
					f.write_str(": ")?;

					member_value.fmt(f)?;
				}

				f.write_str(" }")
			}
			VmVariant::Ref(v) => v.fmt(f),
		}
	}
}

pub trait IntoVariant {
	fn into_variant(self) -> VmVariant;

	// fn clone_into_variant(&self) -> VmVariant
	// where
	// 	Self: Clone,
	// {
	// 	self.clone().into_variant()
	// }
}

impl IntoVariant for String {
	fn into_variant(self) -> VmVariant {
		VmVariant::String(self)
	}
}

impl IntoVariant for Box<str> {
	fn into_variant(self) -> VmVariant {
		VmVariant::String(self.into_string())
	}
}

impl IntoVariant for &str {
	fn into_variant(self) -> VmVariant {
		VmVariant::String(self.to_string())
	}
}

impl IntoVariant for bool {
	fn into_variant(self) -> VmVariant {
		VmVariant::Bool(self)
	}
}

impl IntoVariant for () {
	fn into_variant(self) -> VmVariant {
		VmVariant::Unit
	}

	// fn clone_into_variant(&self) -> VmVariant {
	// 	VmVariant::Unit
	// }
}

impl<T: IntoVariant, const N: usize> IntoVariant for [T; N] {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self.into_iter().map(T::into_variant).collect())
	}
}

impl<T: IntoVariant> IntoVariant for Box<[T]> {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self.into_vec().into_iter().map(T::into_variant).collect())
	}
}

impl<T: IntoVariant + Clone> IntoVariant for &[T] {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self.into_iter().cloned().map(T::into_variant).collect())
	}
}

impl<T: IntoVariant> IntoVariant for Vec<T> {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self.into_iter().map(T::into_variant).collect())
	}
}

impl<const N: usize> IntoVariant for [VmVariant; N] {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self.into())
	}
}

impl IntoVariant for Box<[VmVariant]> {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self.into_vec())
	}
}

impl IntoVariant for &[VmVariant] {
	fn into_variant(self) -> VmVariant {
		let mut vec = Vec::with_capacity(self.len());

		vec.clone_from_slice(self);

		VmVariant::Array(vec)
	}
}

impl IntoVariant for Vec<VmVariant> {
	fn into_variant(self) -> VmVariant {
		VmVariant::Array(self)
	}
}

macro_rules! into_variant_num {
	($($intty:ty),*) => {$(
		impl IntoVariant for $intty {
			fn into_variant(self) -> VmVariant {
				VmVariant::Integer(self as i64)
			}

			// fn clone_into_variant(&self) -> VmVariant {
			// 	VmVariant::Integer((*self) as i64)
			// }
		}
	)*
	};
}

into_variant_num! {
	i8, i16, i32, i64,
	u8, u16, u32, u64
}

pub trait TryFromVariant: Sized {
	fn try_from_variant(variant: VmVariant) -> VmResult<Self>;

	fn expected_vmtype() -> VmType;
}

macro_rules! impl_try_from_variant {
	($($variant:ident => $target:ty),*) => {$(
		impl TryFromVariant for $target {
			fn try_from_variant(variant: VmVariant) -> VmResult<Self> {
				let typeinfo = variant.get_typeinfo();

				if let VmVariant::$variant(v) = variant {
					Ok(v.into())
				} else {
					Err(VmError::invalid_value_type(Self::expected_vmtype().to_string(), typeinfo.to_string()))
				}
			}

			fn expected_vmtype() -> VmType {
				VmType::$variant
			}
		}
	)*};
}

impl_try_from_variant! {
	String => String,
	String => Box<str>,
	Bool => bool,
	Array => Vec<VmVariant>,
	Array => Box<[VmVariant]>,
	Integer => i64
}

// impl TryFromVariant for String {
// 	fn try_from_variant(variant: VmVariant) -> Result<Self> {
// 		let typeinfo = variant.get_typeinfo();

// 		if let VmVariant::String(v) = variant {
// 			Ok(v)
// 		} else {
// 			Err(VmError::InvalidValueType {
// 				expected: Self::expected_vmtype().to_string(),
// 				got: typeinfo.to_string(),
// 			})
// 		}
// 	}

// 	fn expected_vmtype() -> VmType {
// 		VmType::String
// 	}
// }

#[derive(Clone, Debug)]
pub struct StoredValue(Arc<Mutex<VmVariant>>);

impl StoredValue {
	#[inline]
	pub fn new(value: VmVariant) -> Self {
		Self(Arc::new(Mutex::new(value)))
	}

	pub fn value(&self) -> VmVariant {
		self.0.lock().clone()
	}

	pub fn set_value(&self, value: VmVariant) {
		*self.0.lock() = value;
	}
}

impl IntoVariant for StoredValue {
	#[inline]
	fn into_variant(self) -> VmVariant {
		VmVariant::Ref(self)
	}
}

impl IntoVariant for &StoredValue {
	#[inline]
	fn into_variant(self) -> VmVariant {
		VmVariant::Ref(self.clone())
	}
}

impl VmTypable for StoredValue {
	#[inline]
	fn get_typeinfo(&self) -> VmType {
		self.0.lock().get_typeinfo()
	}
}

impl Display for StoredValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.lock().fmt(f)
	}
}

impl From<VmVariant> for StoredValue {
	fn from(value: VmVariant) -> Self {
		Self::new(value)
	}
}
