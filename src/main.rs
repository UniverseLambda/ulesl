mod lexer;
mod parser;
mod vm;
mod common;

use lexer::Lexer;

use crate::{parser::Parser, vm::Vm};


fn main() {
	println!("[VM DEBUG] Hello, world!");

	// let lex = Lexer::new(std::io::stdin(), "stdin".into());
	let lex = Lexer::new(std::fs::File::open("./test.ulesl").unwrap(), "test.ulesl".into());
	let mut parser = Parser::new(lex, "test.ulesl".into());
	let mut vm = Vm::new();

	vm.register_default_builtins();

	loop {
		match parser.next_package() {
			Ok(Some(p)) => {
				println!("[VM DEBUG] Parsed package: {p:?}");

				if let Err(err) = vm.exec_package(p) {
					eprintln!("Vm error: {err:?}");
				}
			},
			Ok(None) => {
				println!("[VM DEBUG] EOF reached!");
				break;
			},
			Err(err) => {
				eprintln!("{err}");
				break;
			}
		}
	}
}
