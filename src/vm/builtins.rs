use std::process::Command;

use crate::common::Location;

use super::{
	error::{VmError, VmResult, VmResultExt},
	types::VmTypable,
	variant::VmVariant,
	Builtin, IntoVariant, Vm,
};

impl Vm {
	pub fn register_builtin(&mut self, name: String, builtin: Builtin) {
		self.builtins.insert(name, builtin);
	}

	pub fn register_default_builtins(&mut self) {
		self.register_builtin("println".to_string(), Self::builtin_println);
		self.register_builtin("exec".to_string(), Self::builtin_exec);
		self.register_builtin("env".to_string(), Self::builtin_env);
		self.register_builtin("typename".to_string(), Self::builtin_typename);
	}

	pub fn builtin_println(&mut self, _name: String, args: Vec<VmVariant>) -> VmResult<VmVariant> {
		if !args.is_empty() {
			print!("{}", args[0]);
		}

		for elem in args.iter().skip(1) {
			print!(" {}", elem);
		}

		println!();

		Ok(VmVariant::Unit)
	}

	pub fn builtin_exec(&mut self, name: String, mut args: Vec<VmVariant>) -> VmResult<VmVariant> {
		if args.is_empty() {
			return Err(
				VmError::wrong_arg_count(1, 0).with_context_func_call(self.caller_location(), name)
			);
		}

		let mut options: Vec<String> = Vec::new();

		for elem in args.remove(0).try_native::<Vec<VmVariant>>()? {
			let VmVariant::String(opt) = elem else {
				return Err(VmError::invalid_value_type(
					"String[]".to_string(),
					"Vary[]".to_string(),
				)
				.with_context_func_arg(self.caller_location(), name, "exec_opt".to_string()));
			};

			options.push(opt);
		}

		if args.is_empty() {
			return Err(
				VmError::wrong_arg_count(2, 1).with_context_func_call(self.caller_location(), name)
			);
		}

		let command: String = args.remove(0).try_native().with_context_func_arg(
			self.caller_location(),
			name.clone(),
			"command".to_string(),
		)?;

		// println!("[VM DEBUG] executing command {command:?} with options {options:?}");

		let mut cmd_builder = Command::new(command);

		for (idx, arg) in args.drain(..).enumerate() {
			let cmd_arg: String = arg.try_native().with_context_func_arg(
				self.caller_location(),
				name.clone(),
				format!("command_arg{idx}"),
			)?;

			cmd_builder.arg(cmd_arg);
		}

		let mut process = match cmd_builder.spawn() {
			Ok(v) => v,
			Err(err) => {
				eprintln!("Could not spawn process: {err}");
				return Ok((-1).into_variant());
			}
		};

		let exit;

		#[cfg(target_family = "unix")]
		{
			use std::os::unix::process::ExitStatusExt;

			exit = process.wait().expect("Command wasn't running").into_raw();
		}
		#[cfg(not(target_family = "unix"))]
		{
			let status = process.wait().expect("Command wasn't running");

			if let Some(code) = status.code() {
				exit = code;
			} else {
				if status.success() {
					exit = 0;
				} else {
					exit = 1;
				}
			}
		}

		Ok(exit.into_variant())
	}

	pub fn builtin_env(
		&mut self,
		func_name: String,
		mut args: Vec<VmVariant>,
	) -> VmResult<VmVariant> {
		if args.len() != 1 {
			return Err(VmError::wrong_arg_count(1, args.len()))
				.with_context_func_call(Location::new_z(0, 0, String::new()), func_name);
		}

		let arg = args.remove(0);

		let env_name: String = arg.try_native().with_context_func_arg(
			self.caller_location(),
			func_name,
			"env_var".to_string(),
		)?;

		match std::env::var(env_name) {
			Ok(v) => Ok(v.into_variant()),
			Err(_) => Ok(VmVariant::Unit),
		}
	}

	pub fn builtin_typename(
		&mut self,
		func_name: String,
		mut args: Vec<VmVariant>,
	) -> VmResult<VmVariant> {
		if args.len() != 1 {
			return Err(VmError::wrong_arg_count(1, args.len()))
				.with_context_func_call(Location::new_z(0, 0, String::new()), func_name);
		}

		Ok(args.remove(0).get_typeinfo().to_string().into_variant())
	}
}
