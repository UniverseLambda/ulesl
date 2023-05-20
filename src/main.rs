mod lexer;
mod parser;
mod vm;
mod common;
use std::io::Read;

use lexer::Lexer;

use crate::{parser::Parser, vm::Vm};

fn main() {
	// println!("[VM DEBUG] Hello, world!");

	let mut args: Vec<String> = std::env::args().skip(1).collect();

	if args.len() > 1 {
		eprintln!("ulesl: Too many arguments");
	}

	let (reader, file): (Box<dyn Read>, String) =
	if args.is_empty() || args[0] == "-" {
		(Box::new(std::io::stdin()), "stdin".into())
	} else {
		(Box::new(std::fs::File::open(&args[0]).expect("ulesl: Could not open input file")), args.pop().unwrap())
	};

	let lex = Lexer::new(reader, file);
	let mut parser = Parser::new(lex, "test.ulesl".into());
	let mut vm = Vm::new();

	vm.register_default_builtins();

	loop {
		match parser.next_package() {
			Ok(Some(p)) => {
				// println!("[VM DEBUG] Parsed package: {p:?}");

				if let Err(err) = vm.exec_package(p) {
					eprintln!("Vm error: {err:?}");
				}
			},
			Ok(None) => {
				// println!("[VM DEBUG] EOF reached!");
				break;
			},
			Err(err) => {
				eprintln!("{err}");
				break;
			}
		}
	}
}
