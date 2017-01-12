use std::fmt;

use ast::{Operator, Type};

use code_generation::utils::*;

#[derive(Debug)]
pub struct CodeGenerator {
    out: Vec<String>,
    last_reg: i32,
    last_label: i32,
    last_str_const: i32,
    br_last_op: bool,
    current_label: Label,
}

impl CodeGenerator {
    pub fn new() -> CodeGenerator {
        let mut cg = CodeGenerator {
            out: Vec::new(),
            last_reg: 0,
            last_label: 0,
            last_str_const: 0,
            br_last_op: false,
            current_label: Label(-1),
        };

        cg.add_func_declare(CGType::new(RawType::TString),
                            &String::from(".concatenate"),
                            &vec![CGType::new(RawType::TString), CGType::new(RawType::TString)]);
        cg.add_func_declare(CGType::new(RawType::TString),
                            &String::from("malloc"),
                            &vec![CGType::new(RawType::TInt)]);
        cg
    }

    pub fn reset(&mut self) {
        self.last_reg = 0;
        self.last_label = 0;
        self.br_last_op = false;
    }

    pub fn get_out(&self) -> &Vec<String> {
        &self.out
    }

    pub fn add_phi(&mut self, t: CGType, op1: (Val, Label), op2: (Val, Label)) -> Register {
        self.new_reg(format!("phi {} [{}, %{}], [{}, %{}]", t, op1.0, op1.1, op2.0, op2.1))
    }

    pub fn add_int_op(&mut self, lhs: Val, op: Operator, rhs: Val) -> Register {
        self.add_op(CGType::new(RawType::TInt), lhs, op, rhs)
    }

    pub fn add_op(&mut self, t: CGType, lhs: Val, op: Operator, rhs: Val) -> Register {
        let op_str = match op {
            Operator::OpAdd => "add",
            Operator::OpSub => "sub",
            Operator::OpMul => "mul",
            Operator::OpDiv => "sdiv",
            Operator::OpMod => "srem",
            Operator::OpEq => "icmp eq",
            Operator::OpLess => "icmp slt",
            Operator::OpLessE => "icmp sle",
            Operator::OpGreater => "icmp sgt",
            Operator::OpGreaterE => "icmp sge",
            _ => unreachable!(),
        };
        self.new_reg(format!("{} {} {}, {}", op_str, t, lhs, rhs))
    }

    pub fn add_neg(&mut self, val: Val) -> Register {
        self.new_reg(format!("sub i32 0, {}", val))
    }

    pub fn add_not(&mut self, val: Val) -> Register {
        self.new_reg(format!("add i1 1, {}", val))
    }

    pub fn add_add_str(&mut self, lhs: Val, rhs: Val) -> Register {
        self.new_reg(format!("call i8* @.concatenate(i8* {}, i8* {})", lhs, rhs))
    }

    pub fn add_loop_step(&mut self, new_idx: Register, old_idx: Register) {
        self.add_line(format!("%{} = add i32 1, %{}", new_idx, old_idx));
    }

    // function
    pub fn add_func_declare(&mut self, ret_type: CGType, func_name: &String, args: &Vec<CGType>) {
        let mut args_str = String::new();
        for arg in args {
            let arg_str = format!("{}", arg);
            args_str = if args_str.is_empty() {
                arg_str
            } else {
                format!("{}, {}", args_str, arg_str)
            };
        }
        self.add_line_no_indent(format!("declare {} @{}({})", ret_type, func_name, args_str));
    }

    pub fn add_func_begin(&mut self,
                          ret_type: CGType,
                          func_name: &String,
                          args: &Vec<CGType>)
                          -> Vec<(Register, CGType)> {
        let mut arg_regs: Vec<(Register, CGType)> = Vec::new();
        let mut args_str = String::new();
        for arg_t in args {
            let reg = self.next_reg();
            let arg_str = format!("{} %{}", arg_t, reg);
            args_str = if args_str.is_empty() {
                arg_str
            } else {
                format!("{}, {}", args_str, arg_str)
            };
            arg_regs.push((reg, *arg_t));
        }

        self.add_line_no_indent(format!("define {} @{}({}) {}",
                                        ret_type,
                                        func_name,
                                        args_str,
                                        '{'));

        let mut arg_addrs: Vec<(Register, CGType)> = Vec::new();
        for arg in &arg_regs {
            let addr_reg = self.add_alloca(arg.1);
            self.add_store(addr_reg, arg.1, Val::Reg(arg.0));
            arg_addrs.push((addr_reg, arg.1));
        }
        arg_addrs
    }

    pub fn add_func_end(&mut self, ret_type: CGType) {
        if ret_type == CGType::new(RawType::TVoid) {
            self.add_line(format!("ret void"));
        }
        self.add_line_no_indent(format!("{}", '}'));
        self.add_line_no_indent(format!(""));
    }

    pub fn add_ret_void(&mut self) {
        self.add_line(format!("ret void"));
    }

    pub fn add_ret(&mut self, t: CGType, val: Val) {
        self.add_line(format!("ret {} {}", t, val));
    }

    pub fn add_call(&mut self,
                    ret_type: CGType,
                    func_name: &String,
                    args: &Vec<(Val, CGType)>)
                    -> Register {
        let mut args_str = String::new();
        for arg in args {
            let arg_str = format!("{} {}", arg.1, arg.0);
            args_str = if args_str.is_empty() {
                arg_str
            } else {
                format!("{}, {}", args_str, arg_str)
            };
        }

        let call_str = format!("call {} @{}({})", ret_type, func_name, args_str);
        if ret_type == CGType::new(RawType::TVoid) {
            self.add_line(call_str);
            self.dummy_reg()
        } else {
            self.new_reg(call_str)
        }
    }

    // memory
    pub fn add_string_constant(&mut self, s: &String) -> StrConstant {
        self.last_str_const += 1;
        let reg = StrConstant(self.last_str_const);
        self.add_line_no_indent(format!("@{} = private unnamed_addr constant [{} x i8] \
                                         c\"{}\\00\"",
                                        reg,
                                        s.len() + 1,
                                        string_to_hex(s)));
        reg
    }

    pub fn add_str_load(&mut self, str_size: usize, str_const: StrConstant) -> Register {
        self.new_reg(format!("getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0",
                             str_size + 1,
                             str_size + 1,
                             str_const))
    }

    pub fn new_arr(&mut self, arr_t: CGType, size: Val) -> Register {
        let struct_ptr = self.add_malloc1(arr_t.llvm_str());
        let arr_ptr = self.add_malloc(format!("{}", arr_t.t), size);

        self.add_comment(format!("Constructing array"));
        let reg = self.new_reg(format!("insertvalue {} undef, i32 {}, 0", arr_t.llvm_str(), size));
        let reg = self.new_reg(format!("insertvalue {} %{}, {}* %{}, 1",
                                       arr_t.llvm_str(),
                                       reg,
                                       arr_t.t,
                                       arr_ptr));
        self.add_comment(format!("Storing array"));
        self.add_raw_store(struct_ptr, arr_t.llvm_str(), reg);
        struct_ptr
    }

    fn add_malloc1(&mut self, t: String) -> Register {
        self.add_malloc(t, Val::Int(1))
    }

    fn add_malloc(&mut self, t: String, size: Val) -> Register {
        self.add_comment(format!("Allocating {}", t));
        let size_of = self.get_sizeof(t.clone(), size);
        let void_addr = self.new_reg(format!("call i8* @malloc(i32 %{})", size_of));
        let cast_addr = self.new_reg(format!("bitcast i8* %{} to {}*", void_addr, t));
        cast_addr
    }

    fn get_sizeof(&mut self, t: String, size: Val) -> Register {
        self.add_comment(format!("Calculating sizeof({})", t));
        let size_of = self.new_reg(format!("getelementptr {}, {}* null, i32 {}", t, t, size));
        self.new_reg(format!("ptrtoint {}* %{} to i32", t, size_of))
    }

    pub fn get_nth_arr_elem(&mut self, struct_ptr: Val, t: CGType, idx: Val) -> (Register, CGType) {
        self.add_comment(format!("Loading array struct"));
        let struct_val = self.add_raw_load(struct_ptr.to_reg(), t.llvm_str());
        self.add_comment(format!("Getting array pointer"));
        let elem0_ptr = self.new_reg(format!("extractvalue {} %{}, 1", t.llvm_str(), struct_val));
        let elem_ptr = self.new_reg(format!("getelementptr {}, {}* %{}, i32 {}",
                                            t.t,
                                            t.t,
                                            elem0_ptr,
                                            idx));
        (elem_ptr, CGType::new(t.t))
    }

    pub fn get_field_addr(&mut self, struct_ptr: Val, t: CGType, idx: i32) -> Register {
        self.new_reg(format!("getelementptr {}, {}* {}, i32 0, i32 {}",
                             t.llvm_str(),
                             t.llvm_str(),
                             struct_ptr,
                             idx))
    }

    pub fn add_alloca(&mut self, t: CGType) -> Register {
        self.new_reg(format!("alloca {}", t))
    }

    pub fn add_load(&mut self, addr_reg: Register, t: CGType) -> Register {
        self.new_reg(format!("load {}, {}* %{}", t, t, addr_reg))
    }

    fn add_raw_load(&mut self, addr_reg: Register, t: String) -> Register {
        self.new_reg(format!("load {}, {}* %{}", t, t, addr_reg))
    }

    pub fn add_store(&mut self, addr_reg: Register, t: CGType, val: Val) {
        self.add_line(format!("store {} {}, {}* %{}", t, val, t, addr_reg));
    }

    fn add_raw_store(&mut self, addr_reg: Register, t: String, val: Register) {
        self.add_line(format!("store {} %{}, {}* %{}", t, val, t, addr_reg));
    }

    // generating registers
    fn new_reg(&mut self, rhs: String) -> Register {
        let reg = self.next_reg();
        self.add_line(format!("%{} = {}", reg, rhs));
        reg
    }

    pub fn next_reg(&mut self) -> Register {
        self.last_reg += 1;
        Register(self.last_reg)
    }

    pub fn dummy_reg(&self) -> Register {
        Register(-100)
    }

    // labels & brs
    pub fn get_current_label(&self) -> Label {
        self.current_label
    }

    pub fn next_label(&mut self) -> Label {
        self.last_label += 1;
        Label(self.last_label)
    }

    pub fn add_label(&mut self, l: Label) {
        self.add_line_no_indent(format!("{}:", l));
        self.current_label = l;
    }

    pub fn add_cond_jump(&mut self, cond: Val, if_true: Label, if_false: Label) {
        if !self.br_last_op {
            self.add_line(format!("br i1 {}, label %{}, label %{}", cond, if_true, if_false));
        }
        self.br_last_op = true;
    }

    pub fn add_jump(&mut self, l: Label) {
        if !self.br_last_op {
            self.add_line(format!("br label %{}", l));
        }
        self.br_last_op = true;
    }

    // core functions
    fn add_comment(&mut self, s: String) {
        self.add_line(format!("; {}", s));
    }

    fn add_line(&mut self, s: String) {
        self.br_last_op = false;
        self.out.push(format!("\t{}", s));
    }

    fn add_line_no_indent(&mut self, s: String) {
        self.br_last_op = false;
        self.out.push(s);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Val {
    Reg(Register),
    Int(i32),
}

impl Val {
    fn to_reg(&self) -> Register {
        match *self {
            Val::Reg(r) => r,
            _ => panic!(),
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Val::Reg(ref r) => write!(f, "%{}", r),
            Val::Int(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(i32);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r_{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StrConstant(i32);

impl fmt::Display for StrConstant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ".str_const_{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Label(i32);

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "label_{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CGType {
    pub is_arr: bool,
    pub t: RawType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawType {
    TInt,
    TBool,
    TVoid,
    TString,
}

impl CGType {
    pub fn new(t: RawType) -> CGType {
        CGType {
            is_arr: false,
            t: t,
        }
    }

    pub fn new_arr(t: RawType) -> CGType {
        CGType {
            is_arr: true,
            t: t,
        }
    }

    pub fn from(t: &Type) -> CGType {
        match *t {
            Type::TArray(ref t) => Self::new_arr(RawType::from(t)),
            _ => Self::new(RawType::from(t)),
        }
    }

    fn ref_for_user(&self) -> bool {
        self.is_arr
    }

    fn llvm_str(&self) -> String {
        if self.is_arr {
            format!("{{ i32, {}* }}", self.t)
        } else {
            format!("{}", self.t)
        }
    }
}

impl fmt::Display for CGType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ref_for_user() {
            write!(f, "{}*", self.llvm_str())
        } else {
            write!(f, "{}", self.llvm_str())
        }
    }
}

impl RawType {
    pub fn from(t: &Type) -> RawType {
        match *t {
            Type::TInt => RawType::TInt,
            Type::TBool => RawType::TBool,
            Type::TString => RawType::TString,
            Type::TVoid => RawType::TVoid,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for RawType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            RawType::TInt => "i32",
            RawType::TBool => "i1",
            RawType::TString => "i8*",
            RawType::TVoid => "void",
        };
        write!(f, "{}", s)
    }
}
