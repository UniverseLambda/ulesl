use super::{Vm, variant::VmVariant, error::Result, Builtin};

impl Vm {
	pub fn register_builtin(&mut self, name: String, builtin: Builtin) {
		self.builtins.insert(name, builtin);
	}

	pub fn register_default_builtins(&mut self) {
		self.register_builtin("println".to_string(), Self::builtin_println)
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
}
