use std::env;
use std::collections::HashMap;
use std::fmt;
use std::io;

#[derive(Clone)]
enum Op {
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

#[derive(Clone)]
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

struct Words {
    dict: HashMap<String, Vec<Op>>,
}

impl Default for Words {
    fn default() -> Self {
        Words {
            dict: HashMap::new(),
        }
    }
}

impl Words {
    fn extend(&mut self, words: HashMap<String, Vec<Op>>) {
        self.dict.extend(words);
    }

    fn find(&self, word: &str) -> Result<Vec<Op>, OpError> {
        self.dict
            .get(word)
            .ok_or(OpError::UndefinedWord)
            .clone()
            .cloned()
    }
}

#[derive(Default)]
struct Machine {
    stack: Vec<i64>,
    words: Words,
}

impl Machine {
    fn exec(&mut self, ops: &Vec<Op>) -> Result<(), OpError> {
        for op in ops {
            match op {
                Op::Num(n) => self.stack.push(*n),
                _ => match op {
                    Op::Add => {
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(a + b);
                    }
                    Op::Mul => {
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(a * b);
                    }
                    Op::Sub => {
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(a - b);
                    }
                    Op::Div => {
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(a / b);
                    }
                    Op::Swap => {
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(b);
                        self.stack.push(a);
                    }
                    Op::Dup => {
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(a);
                        self.stack.push(a);
                    }
                    Op::Over => {
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(a);
                        self.stack.push(b);
                        self.stack.push(a);
                    }
                    Op::Rot => {
                        let c = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let b = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        self.stack.push(b);
                        self.stack.push(c);
                        self.stack.push(a);
                    }
                    Op::Drop => {
                        self.stack.pop().ok_or(OpError::StackUnderflow)?;
                    }
                    Op::Dot => {
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        print!("{}", a);
                    }
                    Op::Emit => {
                        let a = self.stack.pop().ok_or(OpError::StackUnderflow)?;
                        let c =
                            char::from_u32(u32::try_from(a).map_err(|_| OpError::InvalidCharCode)?)
                                .ok_or(OpError::InvalidCharCode)?;
                        print!("{}", c);
                    }
                    Op::Word(word) => {
                        self.exec(&self.words.find(word)?)?;
                    }
                    _ => unreachable!(),
                },
            }
        }
        Ok(())
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
	3 => {
	    match args[1].as_str() {
		"load" => {

		},
		"compile" => {

		},
		_ => print_help()
	    }
	},
	2 => {
	    print_help();
	},
	1 => {
	    repl(Machine::default());
	},
	_ => print_help()
    }
}

fn repl(mut machine: Machine) {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if let Ok((ops, words)) = parser(&input) {
            machine.words.extend(words);
            match machine.exec(&ops) {
                Ok(()) => println!(" ok"),
                Err(err) => println!("error: {}", err),
            }
        }
    }
}

enum Token {
    Num(i64),
    Word(String),
}

impl Token {
    fn is_end_of_word(&self) -> bool {
        match self {
            Token::Num(_) => false,
            Token::Word(word) => word == ";",
        }
    }
}

enum ParserError {
    NoWordName,
}

fn parser(input: &str) -> Result<(Vec<Op>, HashMap<String, Vec<Op>>), ParserError> {
    let mut tokens = input
        .split_whitespace()
        .map(|item| match item.parse::<i64>() {
            Ok(num) => Token::Num(num),
            Err(_) => Token::Word(item.to_string()),
        });

    let mut ops = Vec::new();
    let mut dict = HashMap::new();

    while let Some(token) = tokens.next() {
        match token {
            Token::Num(num) => ops.push(Op::Num(num)),
            Token::Word(word) => {
                if word == ":" {
                    if let Some(Token::Word(word_name)) = tokens.next() {
                        let mut subops = Vec::new();
                        while let Some(token) = tokens.next() {
                            if token.is_end_of_word() {
                                break;
                            } else {
                                subops.push(match token {
                                    Token::Num(num) => Op::Num(num),
                                    Token::Word(word) => parse_word(&word),
                                });
                            }
                        }
                        dict.insert(word_name.to_string(), subops);
                    } else {
                        return Err(ParserError::NoWordName);
                    }
                } else {
                    ops.push(parse_word(&word))
                }
            }
        }
    }
    Ok((ops, dict))
}

fn parse_word(word: &str) -> Op {
    match word {
        "+" => Op::Add,
        "-" => Op::Sub,
        "*" => Op::Mul,
        "/" => Op::Div,
        "." => Op::Dot,
        "emit" => Op::Emit,
        "swap" => Op::Swap,
        "dup" => Op::Dup,
        "over" => Op::Over,
        "rot" => Op::Rot,
        "drop" => Op::Drop,
        word => Op::Word(word.to_string()),
    }
}
