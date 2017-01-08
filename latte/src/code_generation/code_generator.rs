use std::fmt;

use ast::{self, Operator};

#[derive(Debug)]
pub struct CodeGenerator {
    out: Vec<String>,
    last_reg: i32,
    last_label: i32,
    last_str_const: i32,
    br_last_op: bool,
}

impl CodeGenerator {
    pub fn new() -> CodeGenerator {
        let mut cg = CodeGenerator {
            out: Vec::new(),
            last_reg: 0,
            last_label: 0,
            last_str_const: 0,
            br_last_op: false,
        };

        cg.add_func_declare(CGType::TString,
                            &String::from(".concatenate"),
                            &vec![CGType::TString, CGType::TString]);
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

    pub fn add_phi(&mut self,
                   t: CGType,
                   op1: (RegOrInt, Label),
                   op2: (RegOrInt, Label))
                   -> RegOrInt {
        self.new_reg(format!("phi {} [{}, %{}], [{}, %{}]", t, op1.0, op1.1, op2.0, op2.1))
    }

    pub fn add_int_op(&mut self, lhs: RegOrInt, op: Operator, rhs: RegOrInt) -> RegOrInt {
        self.add_op(CGType::TInt, lhs, op, rhs)
    }

    pub fn add_op(&mut self, t: CGType, lhs: RegOrInt, op: Operator, rhs: RegOrInt) -> RegOrInt {
        if let (RegOrInt::Int(l), RegOrInt::Int(r)) = (lhs, rhs) {
            if r != 0 || (op != Operator::OpDiv && op != Operator::OpMod) {
                let res = match op {
                    Operator::OpAdd => l + r,
                    Operator::OpSub => l - r,
                    Operator::OpMul => l * r,
                    Operator::OpDiv => l / r,
                    Operator::OpMod => l % r,
                    Operator::OpEq => (l == r) as i32,
                    Operator::OpLess => (l < r) as i32,
                    Operator::OpLessE => (l <= r) as i32,
                    Operator::OpGreater => (l > r) as i32,
                    Operator::OpGreaterE => (l >= r) as i32,
                    _ => unreachable!(),
                };
                return RegOrInt::Int(res);
            }
        }
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

    pub fn add_neg(&mut self, val: RegOrInt) -> RegOrInt {
        if let RegOrInt::Int(x) = val {
            return RegOrInt::Int(-x);
        }
        self.new_reg(format!("sub i32 0, {}", val))
    }

    pub fn add_not(&mut self, val: RegOrInt) -> RegOrInt {
        if let RegOrInt::Int(x) = val {
            return RegOrInt::Int(1 - x);
        }
        self.new_reg(format!("add i1 1, {}", val))
    }

    pub fn add_add_str(&mut self, lhs: RegOrInt, rhs: RegOrInt) -> RegOrInt {
        self.new_reg(format!("call i8* @.concatenate(i8* {}, i8* {})", lhs, rhs))
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
                          -> Vec<AddrRegister> {
        let mut arg_regs: Vec<(CGType, Register)> = Vec::new();
        let mut args_str = String::new();
        for arg_t in args {
            let reg = self.next_reg();
            let arg_str = format!("{} %{}", arg_t, reg);
            args_str = if args_str.is_empty() {
                arg_str
            } else {
                format!("{}, {}", args_str, arg_str)
            };
            arg_regs.push((*arg_t, reg));
        }

        self.add_line_no_indent(format!("define {} @{}({}) {}",
                                        ret_type,
                                        func_name,
                                        args_str,
                                        '{'));

        let mut arg_addrs: Vec<AddrRegister> = Vec::new();
        for arg in &arg_regs {
            let addr_reg = self.add_alloca(arg.0);
            self.add_store(addr_reg, RegOrInt::Reg(arg.1));
            arg_addrs.push(addr_reg);
        }
        arg_addrs
    }

    pub fn add_func_end(&mut self, ret_type: CGType) {
        if ret_type == CGType::TVoid {
            self.add_line(format!("ret void"));
        }
        self.add_line_no_indent(format!("{}", '}'));
        self.add_line_no_indent(format!(""));
    }

    pub fn add_ret_void(&mut self) {
        self.add_line(format!("ret void"));
    }

    pub fn add_ret(&mut self, t: CGType, val: RegOrInt) {
        self.add_line(format!("ret {} {}", t, val));
    }

    pub fn add_call(&mut self,
                    ret_type: CGType,
                    func_name: &String,
                    args: &Vec<(RegOrInt, CGType)>)
                    -> RegOrInt {
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
        if ret_type == CGType::TVoid {
            self.add_line(call_str);
            RegOrInt::Reg(self.dummy_reg())
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
                                        s));
        reg
    }

    pub fn add_str_load(&mut self, str_size: usize, str_const: StrConstant) -> RegOrInt {
        self.new_reg(format!("getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0",
                             str_size + 1,
                             str_size + 1,
                             str_const))
    }

    pub fn add_alloca(&mut self, t: CGType) -> AddrRegister {
        let reg = self.next_addr_reg(t);
        self.add_line(format!("%{} = alloca {}", reg, t));
        reg
    }

    pub fn add_load(&mut self, addr_reg: AddrRegister) -> RegOrInt {
        let llvm_t = addr_reg.t;
        self.new_reg(format!("load {}, {}* %{}", llvm_t, llvm_t, addr_reg))
    }

    pub fn add_store(&mut self, addr_reg: AddrRegister, val: RegOrInt) {
        let llvm_t = addr_reg.t;
        self.add_line(format!("store {} {}, {}* %{}", llvm_t, val, llvm_t, addr_reg));
    }

    // generating registers
    fn new_reg(&mut self, rhs: String) -> RegOrInt {
        let reg = self.next_reg();
        self.add_line(format!("%{} = {}", reg, rhs));
        RegOrInt::Reg(reg)
    }

    pub fn next_reg(&mut self) -> Register {
        self.last_reg += 1;
        Register(self.last_reg)
    }

    pub fn next_addr_reg(&mut self, t: CGType) -> AddrRegister {
        self.last_reg += 1;
        AddrRegister::new(t, self.last_reg)
    }

    pub fn dummy_reg(&self) -> Register {
        Register(-100)
    }

    // labels & brs
    pub fn next_label(&mut self) -> Label {
        self.last_label += 1;
        Label(self.last_label)
    }

    pub fn add_label(&mut self, l: Label) {
        self.add_line_no_indent(format!("{}:", l));
    }

    pub fn add_cond_jump(&mut self, cond: RegOrInt, if_true: Label, if_false: Label) {
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
pub enum RegOrInt {
    Reg(Register),
    Int(i32),
}

impl fmt::Display for RegOrInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RegOrInt::Reg(ref r) => write!(f, "%{}", r),
            RegOrInt::Int(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(i32);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rv_{}", self.0)
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
pub struct AddrRegister {
    pub t: CGType,
    id: i32,
}

impl AddrRegister {
    pub fn new(t: CGType, id: i32) -> AddrRegister {
        AddrRegister { t: t, id: id }
    }
}

impl fmt::Display for AddrRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ra_{}", self.id)
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
pub enum CGType {
    TInt,
    TBool,
    TString,
    TVoid, // TODO
}

impl CGType {
    pub fn from(t: &ast::Type) -> CGType {
        match *t {
            ast::Type::TInt => CGType::TInt,
            ast::Type::TBool => CGType::TBool,
            ast::Type::TString => CGType::TString,
            ast::Type::TVoid => CGType::TVoid,
            // TODO
            _ => panic!("cannot convert to code generator type"),
        }
    }
}

impl fmt::Display for CGType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            CGType::TInt => "i32",
            CGType::TBool => "i1",
            CGType::TString => "i8*",
            CGType::TVoid => "void",
            // TODO
        };
        write!(f, "{}", s)
    }
}
