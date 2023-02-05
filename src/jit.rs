use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature, UserFuncName},
    isa, settings, Context,
};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use crate::machine::Words;
use crate::Op;
use std::collections::HashMap;

pub type CompiledWord = fn(&mut Vec<i64>) -> ();

pub struct JitCompiler {
    module: JITModule,
    func_ctx: FunctionBuilderContext,
    module_ctx: Context,
    print_func: FuncId,
    push_func: FuncId,
    pop_func: FuncId,
}
impl JitCompiler {
    pub fn new() -> JitCompiler {
        // Everything depends on JITBuilder, which is similar to a program
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        // Define symbols which are on the program but defined externally
        builder.symbol("print_func", print_syscall as *const u8);
        builder.symbol("push_func", push_syscall as *const u8);
        builder.symbol("pop_func", pop_syscall as *const u8);

        // Module is our own code
        let mut module = JITModule::new(builder);
        let mut func_ctx = FunctionBuilderContext::new();
        let mut module_ctx = module.make_context();

        // Signature of print_func for the module
        let mut print_func_sig = module.make_signature();
        print_func_sig.params.push(AbiParam::new(I64));
        print_func_sig.returns.push(AbiParam::new(I64));
        let print_func = module
            .declare_function("print_func", Linkage::Import, &print_func_sig)
            .unwrap();

        let mut push_func_sig = module.make_signature();
        push_func_sig.params.push(AbiParam::new(I64));
        push_func_sig.params.push(AbiParam::new(I64));
        let push_func = module
            .declare_function("push_func", Linkage::Import, &push_func_sig)
            .unwrap();

        let mut pop_func_sig = module.make_signature();
        pop_func_sig.params.push(AbiParam::new(I64));
        pop_func_sig.returns.push(AbiParam::new(I64));
        let pop_func = module
            .declare_function("pop_func", Linkage::Import, &pop_func_sig)
            .unwrap();

        JitCompiler {
            module,
            func_ctx,
            module_ctx,

            print_func,
            push_func,
            pop_func,
        }
    }

    pub fn compile(&mut self, words: Words) -> HashMap<String, CompiledWord> {
        let mut dict = HashMap::new();

        // Compile every word
        for (name, word_ops) in words.dict {
            // Shadow stack is to reduce the use of the real stack inside of a word
            let compiled_word = self.compile_word(&name, word_ops);
            dict.insert(name, compiled_word);
        }
        dict
    }

    fn compile_word(&mut self, name: &str, ops: Vec<Op>) -> CompiledWord {
        let mut shadow_stack = Vec::new();
        // Build a function
        self.module_ctx
            .func
            .signature
            .params
            .push(AbiParam::new(I64));
        let mut builder = FunctionBuilder::new(&mut self.module_ctx.func, &mut self.func_ctx);
        let print_func = self
            .module
            .declare_func_in_func(self.print_func, &mut builder.func);
        let push_func = self
            .module
            .declare_func_in_func(self.push_func, &mut builder.func);
        let pop_func = self
            .module
            .declare_func_in_func(self.pop_func, &mut builder.func);
        // Functions are made of blocks
        let code_block = builder.create_block();
        builder.append_block_params_for_function_params(code_block);
        builder.switch_to_block(code_block);
        let stack = builder.block_params(code_block)[0];

        macro_rules! push {
            ($x:expr) => {
                shadow_stack.push($x);
            };
        }

        macro_rules! pop {
            () => {
                shadow_stack.pop().unwrap_or_else(|| {
                    let pop_call = builder.ins().call(pop_func, &[stack]);
                    builder.inst_results(pop_call)[0]
                })
            };
        }

        macro_rules! flush {
            () => {
                for var in shadow_stack.drain(..) {
                    builder.ins().call(push_func, &[stack, var]);
                }
            };
        }

        for op in ops {
            match op {
                Op::Num(num) => {
                    let a = builder.ins().iconst(I64, num);
                    push!(a);
                }
                Op::Add => {
                    let b = pop!();
                    let a = pop!();
                    let sum = builder.ins().iadd(a, b);
                    push!(sum);
                }
                Op::Sub => {
                    let b = pop!();
                    let a = pop!();
                    let sub = builder.ins().isub(a, b);
                    push!(sub);
                }
                Op::Mul => {
                    let b = pop!();
                    let a = pop!();
                    let mul = builder.ins().imul(a, b);
                    push!(mul);
                }
		Op::Div => {
		    let b = pop!();
		    let a = pop!();
		    let d = builder.ins().sdiv(a, b);
		    push!(d);
		}
                Op::Dot => {
                    let a = pop!();
                    let _print_call = builder.ins().call(print_func, &[a]);
                }
                Op::Dup => {
                    let a = pop!();
                    push!(a);
                    push!(a);
                }
                Op::Swap => {
                    let b = pop!();
                    let a = pop!();
                    push!(b);
                    push!(a);
                }
                Op::Over => {
                    let b = pop!();
                    let a = pop!();
                    push!(a);
                    push!(b);
                    push!(a);
                }
                Op::Rot => {
                    let c = pop!();
                    let b = pop!();
                    let a = pop!();
                    push!(b);
                    push!(c);
                    push!(a);
                }
                Op::Drop => {
                    pop!();
                }
                Op::Word(word) => {
                    flush!();

                    let callee = self
                        .module
                        .declare_function(&word, Linkage::Import, &builder.func.signature)
                        .unwrap();
                    let local_callee = self.module.declare_func_in_func(callee, &mut builder.func);
                    builder.ins().call(local_callee, &[stack]);
                }
		_ => todo!()
            }
        }
        // Flush stack
        flush!();
        // Return
        builder.ins().return_(&[]);

        builder.seal_all_blocks();
        builder.finalize();
        // Finish writing code

        // Compile word and get pointer
        let word = self
            .module
            .declare_function(name, Linkage::Export, &self.module_ctx.func.signature)
            .unwrap();
        self.module
            .define_function(word, &mut self.module_ctx)
            .unwrap();
        self.module.clear_context(&mut self.module_ctx);
        self.module.finalize_definitions().unwrap();
        let raw_code = self.module.get_finalized_function(word);
        let code_ptr = unsafe { std::mem::transmute(raw_code) };
        code_ptr
    }
}

fn print_syscall(value: i64) {
    print!("{}", value);
}

fn push_syscall(stack: &mut Vec<i64>, val: i64) {
    stack.push(val);
}

fn pop_syscall(stack: &mut Vec<i64>) -> i64 {
    stack.pop().unwrap_or(0)
}
