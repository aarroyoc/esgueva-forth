use std::env;
use std::fmt;
use std::io;

mod jit;
mod machine;
mod parser;

use jit::*;
use machine::Machine;

#[derive(Clone)]
pub enum Op {
    Num(i64),
    Add,
    Sub,
    Mul,
    Div,
    Dot,
    Emit,
    Swap,
    Dup,
    Over,
    Rot,
    Drop,
    Word(String),
}

#[derive(Debug, Clone)]
enum OpError {
    StackUnderflow,
    UndefinedWord,
    InvalidCharCode,
}

impl fmt::Display for OpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpError::StackUnderflow => write!(f, "stack underflow"),
            OpError::UndefinedWord => write!(f, "undefined word"),
            OpError::InvalidCharCode => write!(f, "invalid char code"),
        }
    }
}

fn print_help() {
    println!("Usage: esgueva-forth load FILE\t\tStart Esgueva FORTH REPL loading a file");
    println!("       esgueva-forth compile FILE\tCompile a FORTH file to native executable");
    println!("       esgueva-forth \t\t\tStart Esgueva FORTH REPL");
    println!("       esgueva-forth -h\t\t\tShow help");
}

fn main() {
    println!("Esgueva FORTH 0.1.0 - Adrián Arroyo Calle 2023");
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => match args[1].as_str() {
            "load" => {
                let mut machine = Machine::default();
                let (ops, words) = parser::load_from_file(&args[2]);
                machine.words.extend(words.dict);
                machine.exec(&ops).expect("Error executing file");
                repl(machine);
            }
            "compile" => {
                let mut machine = Machine::default();
                let (ops, words) = parser::load_from_file(&args[2]);
                let mut compiler = JitCompiler::new();
                let compiled_words = compiler.compile(words);
                machine.set_jit(compiled_words);
                machine.exec(&ops).expect("Error executing JIT file");
                repl(machine);
            }
            _ => print_help(),
        },
        2 => {
            print_help();
        }
        1 => {
            repl(Machine::default());
        }
        _ => print_help(),
    }
}

fn repl(mut machine: Machine) {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if let Ok((ops, words)) = parser::parser(&input) {
            machine.words.extend(words);
            match machine.exec(&ops) {
                Ok(()) => println!(" ok"),
                Err(err) => println!("error: {}", err),
            }
        }
    }
}
