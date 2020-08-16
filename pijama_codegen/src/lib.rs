use pijama_common::{BinOp, Literal, UnOp};
use pijama_ctx::{Context, ContextExt, LocalId};
use pijama_mir::{PrimFn, RValue, RValueKind, Term, TermKind};
use pijama_ty::Ty;

pub fn run(ctx: &Context, term: &Term) {
    let main = Function::new(0);
    let mut heap = Heap::new();
    heap.push(main);
    let mut compiler = Compiler::new(ctx, &mut heap, 0, LocalId::main());
    compiler.compile(term);
    println!("main: {:?}", compiler.func.chunk);
    *heap.get_mut(0).unwrap() = compiler.func;
    let mut interpreter = Interpreter::new(0, heap);
    interpreter.run();
}

pub fn compile(ctx: &Context, term: &Term) -> Interpreter {
    let main = Function::new(0);
    let mut heap = Heap::new();
    heap.push(main);
    let mut compiler = Compiler::new(ctx, &mut heap, 0, LocalId::main());
    compiler.compile(term);
    println!("main: {:?}", compiler.func.chunk);
    *heap.get_mut(0).unwrap() = compiler.func;
    Interpreter::new(0, heap)
}

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Ptr(usize),
}

impl Value {
    fn assert_int(self) -> i64 {
        if let Self::Int(int) = self {
            int
        } else {
            panic!("expected integer");
        }
    }

    fn assert_ptr(self) -> usize {
        if let Self::Ptr(ptr) = self {
            ptr
        } else {
            panic!("expected pointer");
        }
    }
}

#[derive(Debug, Clone)]
enum OpCode {
    PrintInt,
    PrintBool,
    PrintUnit,
    PrintFunc,
    Not,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shr,
    Shl,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Local(usize),
    Call(usize),
    Push(Value),
    Pop,
    Return,
    JumpIfTrue(usize),
    JumpIfFalse(usize),
    Skip(usize),
}

#[derive(Debug, Clone)]
struct Function {
    arity: usize,
    chunk: Chunk,
}

impl Function {
    fn new(arity: usize) -> Self {
        Self {
            arity,
            chunk: Chunk::default(),
        }
    }

    fn write(&mut self, opcode: OpCode) {
        self.chunk.write(opcode)
    }

    fn read(&mut self, index: usize) -> Option<&OpCode> {
        self.chunk.read(index)
    }
}

type Heap = Vec<Function>;

struct Compiler<'ast, 'ctx, 'heap> {
    ctx: &'ctx Context<'ast>,
    func: Function,
    locals: Vec<LocalId>,
    heap: &'heap mut Heap,
}

impl<'ast, 'ctx, 'heap> Compiler<'ast, 'ctx, 'heap> {
    fn new(ctx: &'ctx Context<'ast>, heap: &'heap mut Heap, ptr: usize, local_id: LocalId) -> Self {
        let func = heap.get(ptr).unwrap().clone();
        Self {
            ctx,
            func,
            locals: vec![local_id],
            heap,
        }
    }

    fn compile_rvalue(&mut self, local_id: LocalId, rvalue: &RValue) {
        // FIXME avoid clone
        match rvalue.kind.clone() {
            RValueKind::Term(kind) => self.compile(&Term {
                id: rvalue.id,
                kind,
            }),
            RValueKind::Abs(ref args, ref body) => {
                let function = Function::new(args.len());
                let ptr = self.heap.len();
                self.heap.push(function);

                let mut compiler = Compiler::new(self.ctx, self.heap, ptr, local_id);
                for arg in args {
                    compiler.locals.push(*arg);
                }
                compiler.compile(body);
                compiler.func.write(OpCode::Return);
                for _ in args {
                    compiler.locals.pop().unwrap();
                }

                let func = compiler.func;
                *self.heap.get_mut(ptr).unwrap() = func;

                self.func.write(OpCode::Push(Value::Ptr(ptr)));
            }
        }
    }

    fn compile(&mut self, term: &Term) {
        match &term.kind {
            TermKind::Lit(lit) => {
                let int = match lit {
                    Literal::Bool(true) => 1,
                    Literal::Bool(false) => 0,
                    Literal::Number(int) => *int,
                    Literal::Unit => 0,
                };
                self.func.write(OpCode::Push(Value::Int(int)));
            }
            TermKind::Var(id) => {
                for (index, id2) in self.locals.iter().enumerate().rev() {
                    if id2 == id {
                        self.func.write(OpCode::Local(index));
                        return;
                    }
                }
                panic!("could not find {:?}", id)
            }
            TermKind::PrimApp(PrimFn::BinOp(BinOp::And), args) => {
                self.compile(&args[0]);
                self.func.write(OpCode::JumpIfFalse(usize::max_value()));

                let start_arg_1 = self.func.chunk.code.len();
                self.func.write(OpCode::Pop);
                self.compile(&args[1]);
                let end_arg_1 = self.func.chunk.code.len();

                self.func.chunk.code[start_arg_1 - 1] =
                    OpCode::JumpIfFalse(end_arg_1 - start_arg_1);
            }
            TermKind::PrimApp(PrimFn::BinOp(BinOp::Or), args) => {
                self.compile(&args[0]);
                self.func.write(OpCode::JumpIfTrue(usize::max_value()));

                let start_arg_1 = self.func.chunk.code.len();
                self.func.write(OpCode::Pop);
                self.compile(&args[1]);
                let end_arg_1 = self.func.chunk.code.len();

                self.func.chunk.code[start_arg_1 - 1] =
                    OpCode::JumpIfTrue(end_arg_1 - start_arg_1);
            }
            TermKind::PrimApp(prim, args) => {
                for arg in args.iter().take(prim.arity()) {
                    self.compile(arg);
                }
                let opcode = match prim {
                    PrimFn::Print => match self.ctx.get_type_info(args[0].id).unwrap().ty {
                        Ty::Int => OpCode::PrintInt,
                        Ty::Bool => OpCode::PrintBool,
                        Ty::Unit => OpCode::PrintUnit,
                        Ty::Arrow(_, _) => OpCode::PrintFunc,
                        Ty::Var(_) => unreachable!(),
                    },
                    PrimFn::UnOp(UnOp::Neg) => OpCode::Neg,
                    PrimFn::UnOp(UnOp::Not) => OpCode::Not,
                    PrimFn::BinOp(BinOp::Add) => OpCode::Add,
                    PrimFn::BinOp(BinOp::Sub) => OpCode::Sub,
                    PrimFn::BinOp(BinOp::Mul) => OpCode::Mul,
                    PrimFn::BinOp(BinOp::Div) => OpCode::Div,
                    PrimFn::BinOp(BinOp::Rem) => OpCode::Rem,
                    PrimFn::BinOp(BinOp::BitAnd) => OpCode::BitAnd,
                    PrimFn::BinOp(BinOp::BitOr) => OpCode::BitOr,
                    PrimFn::BinOp(BinOp::BitXor) => OpCode::BitXor,
                    PrimFn::BinOp(BinOp::Shr) => OpCode::Shr,
                    PrimFn::BinOp(BinOp::Shl) => OpCode::Shl,
                    PrimFn::BinOp(BinOp::Eq) => OpCode::Eq,
                    PrimFn::BinOp(BinOp::Neq) => OpCode::Neq,
                    PrimFn::BinOp(BinOp::Lt) => OpCode::Lt,
                    PrimFn::BinOp(BinOp::Gt) => OpCode::Gt,
                    PrimFn::BinOp(BinOp::Lte) => OpCode::Lte,
                    PrimFn::BinOp(BinOp::Gte) => OpCode::Gte,
                    PrimFn::BinOp(BinOp::And) | PrimFn::BinOp(BinOp::Or) => unreachable!(),
                };
                self.func.write(opcode);
            }
            TermKind::Let(_, lhs_id, rhs, tail) => {
                self.locals.push(*lhs_id);
                self.compile_rvalue(*lhs_id, rhs);
                self.compile(tail);
                self.locals.pop().unwrap();
            }
            TermKind::App(func, args) => {
                self.compile(func);
                for arg in args {
                    self.compile(arg);
                }
                self.func.write(OpCode::Call(args.len()));
            }
            TermKind::Cond(if_term, do_term, else_term) => {
                self.compile(if_term.as_ref());
                self.func.write(OpCode::JumpIfFalse(usize::max_value()));

                let start_do = self.func.chunk.code.len();
                self.func.write(OpCode::Pop);
                self.compile(do_term.as_ref());
                self.func.write(OpCode::Skip(usize::max_value()));
                let end_do = self.func.chunk.code.len();
                self.func.write(OpCode::Pop);

                self.func.chunk.code[start_do - 1] = OpCode::JumpIfFalse(end_do - start_do);

                let start_else = end_do;
                self.compile(else_term.as_ref());
                let end_else = self.func.chunk.code.len();

                self.func.chunk.code[start_else - 1] = OpCode::Skip(end_else - start_else);
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    fn write(&mut self, opcode: OpCode) {
        self.code.push(opcode);
    }

    fn read(&self, index: usize) -> Option<&OpCode> {
        self.code.get(index)
    }
}

#[derive(Clone)]
struct CallFrame {
    function: Function,
    ins_ptr: usize,
    base_ptr: usize,
}

#[derive(Clone)]
struct CallStack {
    head: CallFrame,
    stack: Vec<CallFrame>,
}

impl CallStack {
    fn new(head: CallFrame) -> Self {
        Self {
            head,
            stack: vec![],
        }
    }

    fn head_mut(&mut self) -> &mut CallFrame {
        &mut self.head
    }

    fn head(&self) -> &CallFrame {
        &self.head
    }

    fn push(&mut self, head: CallFrame) {
        let old_head = std::mem::replace(&mut self.head, head);
        self.stack.push(old_head);
    }

    fn pop(&mut self) -> Option<CallFrame> {
        let new_head = self.stack.pop()?;
        Some(std::mem::replace(&mut self.head, new_head))
    }
}

#[derive(Debug, Clone)]
struct ArgStack {
    base_ptr: usize,
    stack: Vec<Value>,
}

impl Default for ArgStack {
    fn default() -> Self {
        Self {
            base_ptr: 0,
            stack: vec![Value::Ptr(0)],
        }
    }
}

impl ArgStack {
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        if self.stack.len() > self.base_ptr {
            Some(self.stack.pop().unwrap())
        } else {
            None
        }
    }

    fn get(&self, index: usize) -> Option<&Value> {
        self.stack.get(index + self.base_ptr)
    }

    fn last(&self) -> Option<&Value> {
        self.stack.last()
    }

    fn len(&self) -> usize {
        self.stack.len() - self.base_ptr
    }

    fn inc_base(&mut self, offset: usize) {
        self.base_ptr += offset;
    }

    fn dec_base(&mut self, offset: usize) {
        self.base_ptr -= offset;
    }

    fn clean(&mut self) {
        self.stack.truncate(self.base_ptr);
    }
}

#[derive(Clone)]
pub struct Interpreter {
    arg_stack: ArgStack,
    call_stack: CallStack,
    heap: Heap,
}

impl Interpreter {
    fn new(main: usize, heap: Heap) -> Self {
        let main = heap.get(main).cloned().unwrap();
        Self {
            call_stack: CallStack::new(CallFrame {
                function: main,
                ins_ptr: 0,
                base_ptr: 0,
            }),
            arg_stack: ArgStack::default(),
            heap,
        }
    }

    fn read_op(&mut self) -> Option<OpCode> {
        let frame = self.call_stack.head_mut();
        let op = frame.function.read(frame.ins_ptr)?.clone();
        frame.ins_ptr += 1;
        Some(op)
    }

    pub fn run(&mut self) {
        println!("{:?}", self.arg_stack);
        while let Some(op) = self.read_op() {
            println!("{:?}", op);
            match op {
                OpCode::PrintInt => {
                    let int = self.arg_stack.pop().unwrap().assert_int();
                    println!("{}", int);
                    self.arg_stack.push(Value::Int(0));
                }
                OpCode::PrintBool => {
                    let int = self.arg_stack.pop().unwrap().assert_int();
                    println!("{}", int != 0);
                    self.arg_stack.push(Value::Int(0));
                }
                OpCode::PrintUnit => {
                    let int = self.arg_stack.pop().unwrap().assert_int();
                    assert_eq!(int, 0);
                    println!("unit");
                    self.arg_stack.push(Value::Int(0));
                }
                OpCode::PrintFunc => {
                    let ptr = self.arg_stack.pop().unwrap().assert_ptr();
                    println!("<function at 0x{:x}>", ptr);
                    self.arg_stack.push(Value::Int(0));
                }
                OpCode::Not => {
                    let int = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int ^ 0));
                }
                OpCode::Neg => {
                    let int = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(-int));
                }
                OpCode::Add => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 + int2));
                }
                OpCode::Sub => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 - int2));
                }
                OpCode::Mul => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 * int2));
                }
                OpCode::Div => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 / int2));
                }
                OpCode::Rem => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 % int2));
                }
                OpCode::And => {
                    // FIXME this need short-circuiting
                    let int2 = self.arg_stack.pop().unwrap().assert_int() != 0;
                    let int1 = self.arg_stack.pop().unwrap().assert_int() != 0;
                    self.arg_stack.push(Value::Int((int1 && int2).into()));
                }
                OpCode::Or => {
                    // FIXME this need short-circuiting
                    let int2 = self.arg_stack.pop().unwrap().assert_int() != 0;
                    let int1 = self.arg_stack.pop().unwrap().assert_int() != 0;
                    self.arg_stack.push(Value::Int((int1 || int2).into()));
                }
                OpCode::BitAnd => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 & int2));
                }
                OpCode::BitOr => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 | int2));
                }
                OpCode::BitXor => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 ^ int2));
                }
                OpCode::Shr => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 >> int2));
                }
                OpCode::Shl => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int(int1 << int2));
                }
                OpCode::Eq => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int((int1 == int2).into()));
                }
                OpCode::Neq => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int((int1 != int2).into()));
                }
                OpCode::Lt => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int((int1 < int2).into()));
                }
                OpCode::Gt => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int((int1 > int2).into()));
                }
                OpCode::Lte => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int((int1 <= int2).into()));
                }
                OpCode::Gte => {
                    let int2 = self.arg_stack.pop().unwrap().assert_int();
                    let int1 = self.arg_stack.pop().unwrap().assert_int();
                    self.arg_stack.push(Value::Int((int1 >= int2).into()));
                }
                OpCode::Push(value) => {
                    let value = value.clone();
                    self.arg_stack.push(value);
                }
                OpCode::Pop => {
                    self.arg_stack.pop().unwrap();
                }
                OpCode::Local(index) => {
                    let value = self.arg_stack.get(index).unwrap().clone();
                    self.arg_stack.push(value);
                }
                OpCode::Call(arity) => {
                    let base_ptr = self.arg_stack.len() - arity - 1;
                    let ptr = self.arg_stack.get(base_ptr).unwrap().clone().assert_ptr();
                    self.arg_stack.inc_base(base_ptr);
                    self.call_stack.push(CallFrame {
                        function: self.heap.get(ptr).cloned().unwrap(),
                        base_ptr,
                        ins_ptr: 0,
                    })
                }
                OpCode::Return => {
                    let ret_value = self.arg_stack.pop().unwrap();
                    let base_ptr = self.call_stack.pop().unwrap().base_ptr;
                    self.arg_stack.clean();
                    self.arg_stack.dec_base(base_ptr);
                    self.arg_stack.push(ret_value);
                }
                OpCode::JumpIfTrue(offset) => {
                    let cond = self.arg_stack.last().unwrap().clone().assert_int();

                    if cond != 0 {
                        self.call_stack.head_mut().ins_ptr += offset;
                    }
                }
                OpCode::JumpIfFalse(offset) => {
                    let cond = self.arg_stack.last().unwrap().clone().assert_int();

                    if cond == 0 {
                        self.call_stack.head_mut().ins_ptr += offset;
                    }
                }
                OpCode::Skip(offset) => {
                    self.call_stack.head_mut().ins_ptr += offset;
                }
            }
            println!("{:?}", self.arg_stack);
        }
    }
}
