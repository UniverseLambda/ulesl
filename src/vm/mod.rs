use std::{cmp::Ordering, collections::HashMap, rc::Rc};

use crate::{
	common::Location,
	parser::types::{
		ArrayExpr, BinaryExpr, BinaryOp, BooleanOperation, Comparison, Expr, FuncCallExpr,
		FuncDecl, IfStatement, LocatedType, NumericalOperation, ParsedHighLevel, VarAssign,
	},
};

use self::{
	error::{VmError, VmResultExt},
	variant::{IntoVariant, VmVariant},
};

mod builtins;
mod error;
mod types;
mod variant;

use error::{VmErrorType, VmResult};
use types::VmTypable;

type Builtin = fn(&mut Vm, String, Vec<VmVariant>) -> VmResult<VmVariant>;

type VmFuncVarAssign<T = VmVariant> = fn(&mut Vm, String, T) -> VmResult<()>;

struct FunctionData {
	packages: Vec<LocatedType<ParsedHighLevel>>,
	args: Vec<String>,
	// return_type: VmType,
}

struct Scope {
	variables: HashMap<String, VmVariant>,
	functions: HashMap<String, Rc<FunctionData>>,
	caller: Location,
}

impl Scope {
	fn new() -> Self {
		Scope {
			variables: HashMap::new(),
			functions: HashMap::new(),
			caller: Location::new_z(0, 0, "_vm".into()),
		}
	}

	fn new_subscope(caller: Location) -> Self {
		Scope {
			variables: HashMap::new(),
			functions: HashMap::new(),
			caller,
		}
	}
}

pub struct Vm {
	global_scope: Scope,
	stack_scope: Option<Scope>,
	builtins: HashMap<String, Builtin>,
	allow_var_shadowing: bool,
	allow_implicit_var: bool,
	root_package_location: Location,
}

impl Vm {
	pub fn new() -> Self {
		Vm {
			global_scope: Scope::new(),
			stack_scope: None,
			builtins: HashMap::new(),
			allow_var_shadowing: false,
			allow_implicit_var: false,
			root_package_location: Location::new_z(0, 0, "_vm".into()),
		}
	}

	pub fn caller_location(&self) -> Location {
		self.get_scope().caller.clone()
	}

	fn get_scope(&self) -> &Scope {
		if let Some(local_scope) = self.stack_scope.as_ref() {
			local_scope
		} else {
			&self.global_scope
		}
	}

	// fn get_scope_mut(&mut self) -> &mut Scope {
	// 	if let Some(local_scope) = self.stack_scope.as_mut() {
	// 		local_scope
	// 	} else {
	// 		&mut self.global_scope
	// 	}
	// }

	pub fn exec_package(
		&mut self,
		located_package: LocatedType<ParsedHighLevel>,
	) -> VmResult<Option<VmVariant>> {
		// let source: String = package.source;

		self.root_package_location = located_package.location;
		let package = located_package.inner;

		let ret = match package {
			ParsedHighLevel::VarDecl(assign_data) => self
				.eval_var_assign(assign_data, Self::new_variable)
				.map(|_| Option::None)?,
			ParsedHighLevel::VarSet(assign_data) => self
				.eval_var_assign(assign_data, Self::set_variable)
				.map(|_| Option::None)?,
			ParsedHighLevel::FuncCall(call_data) => self.eval_func_call(call_data).map(|v| {
				if self.stack_scope.is_none() {
					Option::Some(v)
				} else {
					Option::None
				}
			})?,
			ParsedHighLevel::FuncDecl(func_decl) => {
				self.eval_func_decl(func_decl).map(|_| Option::None)?
			}
			ParsedHighLevel::If(if_statement) => {
				self.eval_if(if_statement).map(|_| Option::None)?
			}
			ParsedHighLevel::Noop => Option::None,
		};

		Ok(ret)
	}

	fn eval_var_assign(&mut self, var_assign: VarAssign, vmfunc: VmFuncVarAssign) -> VmResult<()> {
		let evaluated_val = self.eval_expr(var_assign.val)?;

		vmfunc(self, var_assign.name, evaluated_val)
	}

	fn eval_func_call(&mut self, mut func_call_expr: FuncCallExpr) -> VmResult<VmVariant> {
		let mut params: Vec<VmVariant> = Vec::with_capacity(func_call_expr.args.len());

		for arg_expr in func_call_expr.args.drain(..) {
			params.push(self.eval_expr(arg_expr)?);
		}

		self.call_func(func_call_expr.name, params)
	}

	fn eval_func_decl(&mut self, func_decl: FuncDecl) -> VmResult<()> {
		let scope = if let Some(scope) = self.stack_scope.as_mut() {
			scope
		} else {
			&mut self.global_scope
		};

		let (name, func_data) = func_decl.into();

		if scope.functions.contains_key(&name) {
			return Err(VmError::func_name_dup(name));
		}

		scope.functions.insert(name, Rc::new(func_data));

		Ok(())
	}

	fn eval_if(&mut self, mut if_statement: IfStatement) -> VmResult<()> {
		let cond_variant = self.eval_expr(if_statement.val)?;

		if cond_variant.try_native()? {
			let old_scope = self.stack_scope.take();

			self.stack_scope = Some(Scope::new());

			for package in if_statement.block.statements.drain(..) {
				self.exec_package(package)?;
			}

			self.stack_scope = old_scope;
		}

		Ok(())
	}

	fn eval_array(&mut self, mut array_data: ArrayExpr) -> VmResult<VmVariant> {
		let elems: Vec<VmVariant> = array_data
			.args
			.drain(..)
			.map(|e| self.eval_expr(e))
			.collect::<VmResult<Vec<VmVariant>>>()?;

		Ok(VmVariant::Array(elems))
	}

	fn eval_expr(&mut self, expr: Expr) -> VmResult<VmVariant> {
		Ok(match expr {
			Expr::IntLiteral(v) => VmVariant::Integer(v),
			Expr::StringLiteral(v) => VmVariant::new_from_string_expr(&v)?,
			Expr::BoolLiteral(v) => VmVariant::Bool(v),
			Expr::Identifier(var_name) => self.get_variable(&var_name)?,
			Expr::FuncCall(call_data) => self.eval_func_call(call_data)?,
			Expr::Array(array_data) => self.eval_array(array_data)?,
			Expr::Binary(compare_data) => self.eval_binary_expr(compare_data)?,
		})
	}

	fn eval_binary_expr(&mut self, expr: BinaryExpr) -> VmResult<VmVariant> {
		match expr.op {
			BinaryOp::Compare(op) => {
				let left = self.eval_expr(*expr.left)?;
				let right = self.eval_expr(*expr.right)?;

				self.eval_comparison(op, left, right)
			}
			BinaryOp::Bool(op) => self.eval_bool_op(op, *expr.left, *expr.right),
			BinaryOp::Numerical(op) => {
				let left = self.eval_expr(*expr.left)?;
				let right = self.eval_expr(*expr.right)?;

				self.eval_numerical_op(op, left, right)
			}
		}
	}

	fn eval_comparison(
		&mut self,
		op: Comparison,
		left: VmVariant,
		right: VmVariant,
	) -> VmResult<VmVariant> {
		let Some(ord) = left.compare(&right) else {
			return Err(VmError::new(VmErrorType::InvalidComparison {
				left_type: left.get_typeinfo().to_string(),
				right_type: right.get_typeinfo().to_string(),
			}));
		};

		Ok(match (ord, op) {
			(
				Ordering::Equal,
				Comparison::Equal | Comparison::GreaterOrEqual | Comparison::LessOrEqual,
			) => VmVariant::TRUE,
			(ord, Comparison::NotEqual) => VmVariant::Bool(ord != Ordering::Equal),
			(Ordering::Greater, Comparison::Greater | Comparison::GreaterOrEqual) => {
				VmVariant::TRUE
			}
			(Ordering::Less, Comparison::Less | Comparison::LessOrEqual) => VmVariant::TRUE,
			_ => VmVariant::FALSE,
		})
	}

	fn eval_bool_op(
		&mut self,
		op: BooleanOperation,
		left: Expr,
		right: Expr,
	) -> VmResult<VmVariant> {
		let left: bool = self.eval_expr(left)?.try_native()?;

		match (op, left) {
			(BooleanOperation::Or, false) | (BooleanOperation::And, true) => self
				.eval_expr(right)?
				.try_native::<bool>()
				.map(VmVariant::Bool),
			(BooleanOperation::Or, v) | (BooleanOperation::And, v) => Ok(VmVariant::Bool(v)),
		}
	}

	fn eval_numerical_op(
		&mut self,
		op: NumericalOperation,
		left: VmVariant,
		right: VmVariant,
	) -> VmResult<VmVariant> {
		let left: i64 = left.try_native()?;
		let right: i64 = right.try_native()?;

		Ok(VmVariant::from(match op {
			NumericalOperation::Add => left + right,
			NumericalOperation::Sub => left - right,
			NumericalOperation::Mul => left * right,
			NumericalOperation::Div => left / right,
		}))
	}

	pub fn new_variable<T: Into<VmVariant>>(&mut self, var_name: String, value: T) -> VmResult<()> {
		let scope = if let Some(scope) = self.stack_scope.as_mut() {
			scope
		} else {
			&mut self.global_scope
		};

		if !self.allow_var_shadowing && scope.variables.contains_key(&var_name) {
			Err(VmError::var_name_dup(var_name))
		} else {
			let vm_value: VmVariant = value.into();

			// println!("[VM DEBUG] New variable: \"{}\" (value: {:?})", var_name, vm_value);

			scope.variables.insert(var_name, vm_value);

			Ok(())
		}
	}

	pub fn set_variable<T: Into<VmVariant>>(&mut self, var_name: String, value: T) -> VmResult<()> {
		if !self.allow_implicit_var
			&& self
				.stack_scope
				.as_ref()
				.map_or(false, |scope| scope.variables.contains_key(&var_name))
			&& !self.global_scope.variables.contains_key(&var_name)
		{
			return Err(VmError::unknown_identifier(var_name));
		}

		let vm_value: VmVariant = value.into();

		// println!("[VM DEBUG] Variable update: \"{}\" (new value: {:?})", var_name, vm_value);

		if let Some(scope) = self.stack_scope.as_mut() {
			scope
		} else {
			&mut self.global_scope
		}
		.variables
		.insert(var_name, vm_value);

		Ok(())
	}

	pub fn get_variable(&self, var_name: &String) -> VmResult<VmVariant> {
		let scope = if let Some(scope) = self.stack_scope.as_ref() {
			scope
		} else {
			&self.global_scope
		};

		// TODO: Also check global scope if stack_scope

		if let Some(val) = scope.variables.get(var_name) {
			Ok(val.clone())
		} else {
			Err(VmError::unknown_identifier(var_name.clone()))
		}
	}

	pub fn call_func(
		&mut self,
		func_name: String,
		mut params: Vec<VmVariant>,
	) -> VmResult<VmVariant> {
		if let Some(builtin_func) = self.builtins.get(&func_name) {
			return builtin_func(self, func_name, params);
		}

		let user_func = {
			if let Some(scope) = self.stack_scope.as_mut() {
				scope
			} else {
				&self.global_scope
			}
			.functions
			.get(&func_name)
			.cloned()
		};

		// println!("[VM DEBUG] Trying to call {} with params {:?}", func_name, params);

		if let Some(user_func) = user_func {
			let old_scope = self.stack_scope.take();

			self.stack_scope = Some(Scope::new_subscope(Location::new_z(
				0,
				0,
				"_vm".to_string(),
			)));

			// TODO: Parameters

			if params.len() != user_func.args.len() {
				return Err(VmError::wrong_arg_count(user_func.args.len(), params.len()))
					.with_context_func_call(self.caller_location(), func_name);
			}

			let zipped = user_func.args.iter().zip(params.drain(..));

			for (name, value) in zipped {
				self.stack_scope
					.as_mut()
					.unwrap()
					.variables
					.insert(name.clone(), value);
			}

			// zipped.unzip() when I'll implement default values

			let mut res: VmResult<VmVariant> = Ok(VmVariant::Unit);

			for package in &user_func.packages {
				match self.exec_package(package.clone()) {
					Ok(Some(v)) => {
						res = Ok(v);
						break;
					}
					Err(err) => {
						res = Err(err);
						break;
					}

					Ok(None) => (),
				};
			}

			// TODO: Properly clean previous stack scope (when type cleanup is implemented, of course)

			self.stack_scope = old_scope;

			return res;
		}

		Err(VmError::unknown_identifier(func_name))
	}
}
