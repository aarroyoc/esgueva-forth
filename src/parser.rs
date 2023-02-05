use std::collections::HashMap;
use std::fs;

use crate::machine::Words;
use crate::Op;

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

pub fn load_from_file(file: &str) -> (Vec<Op>, Words) {
    let mut words = Words::default();
    let contents = fs::read_to_string(file).expect("File must exist");
    let ops = contents
        .lines()
        .map(|line| parser(line))
        .map(|parse_result| {
            if let Ok((ops, words1)) = parse_result {
                words.extend(words1);
                ops
            } else {
                vec![]
            }
        })
        .collect::<Vec<Vec<Op>>>()
        .concat();
    (ops, words)
}

pub enum ParserError {
    NoWordName,
}

pub fn parser(input: &str) -> Result<(Vec<Op>, HashMap<String, Vec<Op>>), ParserError> {
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
