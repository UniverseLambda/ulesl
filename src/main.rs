mod lexer;
use lexer::Lexer;

use crate::{parser::Parser, vm::Vm};

mod parser;

mod vm;

fn main() {
	println!("Hello, world!");

	// let lex = Lexer::new(std::io::stdin(), "stdin".into());
	let lex = Lexer::new(std::fs::File::open("./test.ulesl").unwrap(), "test.ulesl".into());
	let mut parser = Parser::new(lex, "test.ulesl".into());
	let mut vm = Vm::new();

	vm.register_default_builtins();

	loop {
		match parser.next_package() {
			Ok(Some(p)) => {
				println!("Parsed package: {:?}", p);

				if let Err(err) = vm.exec_package(p) {
					eprintln!("Vm error: {:?}", err);
				}
			},
			Ok(None) => {
				println!("EOF reached!");
				break;
			},
			Err(err) => {
				println!("Parser error: {:?}", err);
				break;
			}
		}
	}
}
