use std::{collections::HashMap, rc::Rc};

use crate::parser::{ParsedPackage, Expr, ParsedHighLevel, VarAssign, FuncCallExpr, FuncDecl, ArrayExpr};

use self::{variant::VmVariant, types::VmType, error::VmError};

mod builtins;
mod error;
mod types;
mod variant;

use error::Result;

type Builtin = fn (&mut Vm, String, Vec<VmVariant>) -> Result<VmVariant>;

type VmFuncVarAssign<T: Into<VmVariant> = VmVariant> = fn (&mut Vm, String, T) -> Result<()>;

struct FunctionData {
	packages: Vec<ParsedPackage>,
	return_type: VmType,
}

struct Scope {
	variables: HashMap<String, VmVariant>,
	functions: HashMap<String, Rc<FunctionData>>,
}

impl Scope {
	fn new() -> Self {
		Scope {
			variables: HashMap::new(),
			functions: HashMap::new(),
		}
	}
}

pub struct Vm {
	global_scope: Scope,
	stack_scope: Option<Scope>,
	builtins: HashMap<String, Builtin>,
	allow_var_shadowing: bool,
	allow_implicit_var: bool,
}

impl Vm {
	pub fn new() -> Self {
		Vm {
			global_scope: Scope::new(),
			stack_scope: None,
			builtins: HashMap::new(),
			allow_var_shadowing: false,
			allow_implicit_var: false,
		}
	}

	pub fn exec_package(&mut self, package: ParsedPackage) -> Result<Option<VmVariant>> {
		// let source: String = package.source;

		let ret = match package.parsed {
			ParsedHighLevel::VarDecl(assign_data) => self.eval_var_assign(assign_data, Self::new_variable).map(|_| Option::None)?,
			ParsedHighLevel::VarSet(assign_data) => self.eval_var_assign(assign_data, Self::set_variable).map(|_| Option::None)?,
			ParsedHighLevel::FuncCall(call_data) => self.eval_func_call(call_data).map(|v| if let None = self.stack_scope { Option::Some(v) } else { Option::None } )?,
			ParsedHighLevel::FuncDecl(func_decl) => self.eval_func_decl(func_decl).map(|_| Option::None)?,
		};

		Ok(ret)
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

	fn eval_func_decl(&mut self, func_decl: FuncDecl) -> Result<()> {
		let scope = if let Some(scope) = self.stack_scope.as_mut() {
			scope
		} else {
			&mut self.global_scope
		};

		let (name, func_data) = func_decl.into();

		if scope.functions.contains_key(&name) {
			return Err(VmError::FuncNameDuplicate(name));
		}

		scope.functions.insert(name, Rc::new(func_data));

		Ok(())
	}

	fn eval_array(&mut self, mut array_data: ArrayExpr) -> Result<VmVariant> {
		let elems: Vec<VmVariant> = array_data.args.drain(..).map(|e| {
			self.eval_expr(e)
		}).collect::<Result<Vec<VmVariant>>>()?;

		Ok(VmVariant::Array(elems))
	}

	fn eval_expr(&mut self, expr: Expr) -> Result<VmVariant> {
		Ok(match expr {
			Expr::IntLiteral(v) => VmVariant::Integer(v),
			Expr::StringLiteral(v) => VmVariant::String(v),
			Expr::Identifier(var_name) => self.get_variable(&var_name)?,
			Expr::FuncCall(call_data) => self.eval_func_call(call_data)?,
			Expr::Array(array_data) => self.eval_array(array_data)?,
		})
	}

	pub fn new_variable<T: Into<VmVariant>>(&mut self, var_name: String, value: T) -> Result<()> {
		let scope = if let Some(scope) = self.stack_scope.as_mut() {
			scope
		} else {
			&mut self.global_scope
		};

		if !self.allow_var_shadowing && scope.variables.contains_key(&var_name) {
			Err(VmError::VarNameDuplicate(var_name))
		} else {
			let vm_value: VmVariant = value.into();

			println!("[VM DEBUG] New variable: \"{}\" (value: {:?})", var_name, vm_value);

			scope.variables.insert(var_name, vm_value);

			Ok(())
		}
	}

	pub fn set_variable<T: Into<VmVariant>>(&mut self, var_name: String, value: T) -> Result<()> {
		if !self.allow_implicit_var {
			if self.stack_scope.as_ref().map_or(
				false, |scope| scope.variables.contains_key(&var_name))
				&& !self.global_scope.variables.contains_key(&var_name) {
				return Err(VmError::VarNameNotFound(var_name));
			}
		}

		let vm_value: VmVariant = value.into();

		println!("[VM DEBUG] Variable update: \"{}\" (new value: {:?})", var_name, vm_value);

		if let Some(scope) = self.stack_scope.as_mut() {
			scope
		} else {
			&mut self.global_scope
		}.variables.insert(var_name, vm_value);

		Ok(())
	}

	pub fn get_variable(&self, var_name: &String) -> Result<VmVariant> {
		let scope = if let Some(scope) = self.stack_scope.as_ref() {
			scope
		} else {
			&self.global_scope
		};

		// TODO: Also check global scope if stack_scope

		if let Some(val) = scope.variables.get(var_name) {
			Ok(val.clone())
		} else {
			Err(VmError::VarNameNotFound(var_name.clone()))
		}
	}

	pub fn call_func(&mut self, func_name: String, params: Vec<VmVariant>) -> Result<VmVariant> {
		if let Some(builtin_func) = self.builtins.get(&func_name) {
			return builtin_func(self, func_name, params);
		}

		let user_func = {
			if let Some(scope) = self.stack_scope.as_mut() {
				scope
			} else {
				&self.global_scope
			}.functions.get(&func_name).cloned()
		};

		println!("[VM DEBUG] Trying to call {} with params {:?}", func_name, params);

		if let Some(user_func) = user_func {
			let old_scope = self.stack_scope.take();

			self.stack_scope = Some(Scope::new());

			let mut res: Result<VmVariant> = Ok(VmVariant::Unit);

			for package in &user_func.packages {
				match self.exec_package(package.clone()) {
					Ok(Some(v)) => { res = Ok(v); break; },
					Err(err) => { res = Err(err); break; },

					Ok(None) => (),
				};
			};

			// TODO: Properly clean previous stack scope (when type cleanup is implemented, of course)

			self.stack_scope = old_scope;

			return res;
		}

		Err(VmError::FuncNameNotFound(func_name))
	}
}
