mod common;
mod lexer;
mod parser;
mod vm;
use std::io::{IsTerminal, Read, Write};

use lexer::Lexer;

use crate::{parser::Parser, vm::Vm};

fn main() {
	// println!("[VM DEBUG] Hello, world!");

	let mut args: Vec<String> = std::env::args().skip(1).collect();

	if args.len() > 1 {
		eprintln!("ulesl: Too many arguments");
	}

	let (reader, file, interactive): (Box<dyn Read>, String, bool) =
		if args.is_empty() || args[0] == "-" {
			(
				Box::new(std::io::stdin()),
				"stdin".into(),
				std::io::stdin().is_terminal(),
			)
		} else {
			(
				Box::new(std::fs::File::open(&args[0]).expect("ulesl: Could not open input file")),
				args.pop().unwrap(),
				false,
			)
		};

	let lex = Lexer::new(reader, file);
	let mut parser = Parser::new(lex, "test.ulesl".into());
	let mut vm = Vm::new();

	vm.register_default_builtins();

	loop {
		if interactive {
			print!("ulesl> ");
			let _ = std::io::stdout().flush();
		}

		match parser.next_package() {
			Ok(Some(p)) => {
				// println!("[VM DEBUG] Parsed package: {p:?}");

				if let Err(err) = vm.exec_package(p) {
					eprintln!("Vm error: {err:?}");
				}
			}
			Ok(None) => {
				// println!("[VM DEBUG] EOF reached!");
				break;
			}
			Err(err) => {
				eprintln!("{err}");
				if !interactive {
					break;
				}
			}
		}
	}
}
