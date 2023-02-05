use std::collections::HashMap;

use crate::jit::CompiledWord;
use crate::{Op, OpError};

pub struct Words {
    pub dict: HashMap<String, Vec<Op>>,
}

impl Default for Words {
    fn default() -> Self {
        Words {
            dict: HashMap::new(),
        }
    }
}

impl Words {
    pub fn extend(&mut self, words: HashMap<String, Vec<Op>>) {
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
pub struct Machine {
    stack: Vec<i64>,
    pub words: Words,
    compiled_words: HashMap<String, CompiledWord>,
}

impl Machine {
    pub fn set_jit(&mut self, compiled_words: HashMap<String, CompiledWord>) {
        self.compiled_words = compiled_words;
    }
    pub fn exec(&mut self, ops: &Vec<Op>) -> Result<(), OpError> {
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
                        if let Some(compiled_word) = self.compiled_words.get(word) {
                            compiled_word(&mut self.stack);
                        } else {
                            self.exec(&self.words.find(word)?)?;
                        }
                    }
                    _ => unreachable!(),
                },
            }
        }
        Ok(())
    }
}
