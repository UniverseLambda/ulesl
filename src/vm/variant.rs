/**
 * This file is part of EasyScriptingLanguage.
 *
 * EasyScriptingLanguage is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * EasyScriptingLanguage is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with EasyScriptingLanguage.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::parser;

trait VmTypable {
	fn get_typeinfo(&self) -> VmType;
}

pub enum VmType {
	// Variant,
	Unit,
	Integer,
	String,
	// ReadStream,
	// WriteStream,
	Array,
}

#[derive(Clone, Debug)]
pub enum VmVariant {
	Unit,
	Integer(i64),
	String(String),
	// ReadStream(Box<dyn Read>),
	// WriteStream(Box<dyn Write>),
	Array(Vec<VmVariant>),
}

impl VmTypable for VmVariant {
	fn get_typeinfo(&self) -> VmType {
		match self {
			VmVariant::Unit => VmType::Unit,
			VmVariant::Integer(_) => VmType::Integer,
			VmVariant::String(_) => VmType::String,
			VmVariant::Array(_) => VmType::Array,
		}
	}
}

impl From<parser::Expr> for VmVariant {
	fn from(value: parser::Expr) -> Self {
		match value {
			parser::Expr::IntLiteral(v) => Self::Integer(v),
			parser::Expr::StringLiteral(v) => Self::String(v),
			parser::Expr::Identifier(_) => todo!(),
			parser::Expr::FuncCall(_) => todo!(),
		}
	}
}
