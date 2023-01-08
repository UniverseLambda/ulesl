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

use std::collections::HashMap;

use crate::parser::{ParsedPackage, Expr, ParsedHighLevel, VarAssign, FuncCallExpr};

use self::variant::VmVariant;

mod error;
mod builtins;
mod variant;

use error::Result;

type Builtin = fn (&mut Vm, String, Vec<String>) -> Result<VmVariant>;
// type ExitStatus = u32;

type VmFuncVarAssign<T: Into<VmVariant> = VmVariant> = fn (&mut Vm, String, T) -> Result<()>;

pub struct Vm {
	variables: HashMap<String, VmVariant>,
	builtins: HashMap<String, Builtin>,
	allow_var_shadowing: bool,
	allow_implicit_var: bool,
}

impl Vm {
	pub fn new() -> Self {
		Vm {
			variables: HashMap::new(),
			builtins: HashMap::new(),
			allow_var_shadowing: false,
			allow_implicit_var: false,
		}
	}

	pub fn exec_package(&mut self, package: ParsedPackage) -> Result<()> {
		// let source: String = package.source;

		match package.parsed {
			ParsedHighLevel::VarDecl(assign_data) => self.eval_var_assign(assign_data, Self::new_variable)?,
			ParsedHighLevel::VarSet(assign_data) => self.eval_var_assign(assign_data, Self::set_variable)?,
			ParsedHighLevel::FuncCall(call_data) => { self.eval_func_call(call_data)?; () },
		};

		Ok(())
	}

	fn eval_var_assign(&mut self, var_assign: VarAssign, vmfunc: VmFuncVarAssign) -> Result<()> {
		let evaluated_val = self.eval_expr(var_assign.val)?;

		vmfunc(self, var_assign.name, evaluated_val)
	}

	fn eval_func_call(&mut self, mut func_call_expr: FuncCallExpr) -> Result<VmVariant> {
		let mut params: Vec<VmVariant> = Vec::with_capacity(func_call_expr.args.len());

		for arg_expr in func_call_expr.args.drain(..) {
			params.push(self.eval_expr(arg_expr)?);
		}

		self.call_func(func_call_expr.name, params)
	}

	fn eval_expr(&mut self, expr: Expr) -> Result<VmVariant> {
		Ok(match expr {
			Expr::IntLiteral(v) => VmVariant::Integer(v),
			Expr::StringLiteral(v) => VmVariant::String(v),
			Expr::Identifier(var_name) => self.get_variable(&var_name)?,
			Expr::FuncCall(call_data) => self.eval_func_call(call_data)?,
		})
	}

	pub fn new_variable<T: Into<VmVariant>>(&mut self, var_name: String, value: T) -> Result<()> {
		if !self.allow_var_shadowing && self.variables.contains_key(&var_name) {
			Err(())
		} else {
			let vm_value: VmVariant = value.into();

			println!("[VM DEBUG] New variable: \"{}\" (value: {:?})", var_name, vm_value);

			self.variables.insert(var_name, vm_value);

			Ok(())
		}
	}

	pub fn set_variable<T: Into<VmVariant>>(&mut self, var_name: String, value: T) -> Result<()> {
		if !self.allow_implicit_var && !self.variables.contains_key(&var_name) {
			Err(())
		} else {
			let vm_value: VmVariant = value.into();

			println!("[VM DEBUG] Variable update: \"{}\" (new value: {:?})", var_name, vm_value);

			self.variables.insert(var_name, vm_value);


			Ok(())
		}
	}

	pub fn get_variable(&self, var_name: &String) -> Result<VmVariant> {
		if let Some(val) = self.variables.get(var_name) {
			Ok(val.clone())
		} else {
			Err(())
		}
	}

	pub fn call_func(&mut self, func_name: String, params: Vec<VmVariant>) -> Result<VmVariant> {
		println!("[VM DEBUG] Trying to call {} with params {:?}", func_name, params);

		Err(())
	}
}
