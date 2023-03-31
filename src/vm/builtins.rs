use std::process::Command;

use super::{Vm, variant::VmVariant, error::{Result, VmError}, Builtin, types::VmType};

impl Vm {
	pub fn register_builtin(&mut self, name: String, builtin: Builtin) {
		self.builtins.insert(name, builtin);
	}

	pub fn register_default_builtins(&mut self) {
		self.register_builtin("println".to_owned(), Self::builtin_println);
		self.register_builtin("exec".to_owned(), Self::builtin_exec);
	}

	pub fn builtin_println(&mut self, _name: String, args: Vec<VmVariant>) -> Result<VmVariant> {
		if !args.is_empty() {
			print!("{}", args[0]);
		}

		for elem in args.iter().skip(1) {
			print!(" {}", elem);
		}

		println!();

		Ok(VmVariant::Unit)
	}

	pub fn builtin_exec(&mut self, name: String, mut args: Vec<VmVariant>) -> Result<VmVariant> {
		if args.is_empty() {
			return Err(VmError::NotEnoughArg {func_name: name, expected: 1, got: 0});
		}

		let mut options: Vec<String> = Vec::new();

		if let VmVariant::Array(array) = args.remove(0) {
			for elem in array {
				let VmVariant::String(opt) = elem else {
					return Err(VmError::InvalidArgType { func_name: name, arg_name: "exec_opt".to_owned(), expected: "String[]".to_owned(), got: "Vary[]".to_owned() })
				};

				options.push(opt);
			}
		}

		if args.is_empty() {
			return Err(VmError::NotEnoughArg {func_name: name, expected: 2, got: 1});
		}

		self.expect_variant_type(&name, "command", args.first().unwrap(), VmType::String)?;

		let command = args.remove(0).unwrap_string();

		// println!("[VM DEBUG] executing command {command:?} with options {options:?}");

		let mut cmd_builder = Command::new(command);


		for (idx, arg) in args.drain(..).enumerate() {
			self.expect_variant_type(&name, &format!("command_arg{idx}"), &arg, VmType::String)?;

			let cmd_arg = arg.unwrap_string();

			cmd_builder.arg(cmd_arg);
		}

		let mut process = match cmd_builder.spawn() {
			Ok(v) => v,
			Err(err) => {
				eprintln!("Could not spawn process: {err}");
				return Ok(VmVariant::Integer(-1));
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


		Ok(VmVariant::Integer(exit as i64))
	}
}
