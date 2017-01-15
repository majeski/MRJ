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

        cg.add_comment(format!("internal functions"));
        cg.add_func_declare(CGType::new(RawType::TRawPtr),
                            &format!(".concatenate"),
                            &vec![CGType::new(RawType::TRawPtr), CGType::new(RawType::TRawPtr)]);
        cg.add_func_declare(CGType::new(RawType::TVoid),
                            &format!("._retain_str"),
                            &vec![CGType::new(RawType::TString)]);
        cg.add_func_declare(CGType::new(RawType::TVoid),
                            &format!("._release_str"),
                            &vec![CGType::new(RawType::TString)]);
        cg.add_func_declare(CGType::new(RawType::TVoid),
                            &format!("._init_str_arr"),
                            &vec![CGType::new_arr(RawType::TString)]);
        cg.add_func_declare(CGType::new(RawType::TRawPtr),
                            &format!("malloc"),
                            &vec![CGType::new(RawType::TInt)]);
        cg.add_empty_line();
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
        let str_t = CGType::new(RawType::TString);
        self.add_comment(format!("Getting lhs string"));
        let lhs_str_ptr = self.get_field_addr(lhs, str_t, 1);
        let lhs_str = self.add_raw_load(lhs_str_ptr, format!("i8*"));
        self.add_comment(format!("Getting rhs string"));
        let rhs_str_ptr = self.get_field_addr(rhs, str_t, 1);
        let rhs_str = self.add_raw_load(rhs_str_ptr, format!("i8*"));
        self.add_comment(format!("Concatenate"));
        let new_str =
            self.new_reg(format!("call i8* @.concatenate(i8* %{}, i8* %{})", lhs_str, rhs_str));

        let struct_ptr = self.alloc_string();
        self.retain_string(Val::Reg(struct_ptr));
        let str_ptr_addr = self.get_field_addr(Val::Reg(struct_ptr), str_t, 1);
        self.add_raw_store(str_ptr_addr, format!("i8*"), Val::Reg(new_str));
        struct_ptr
    }

    pub fn add_loop_step(&mut self, new_idx: Register, old_idx: Register) {
        self.add_line(format!("%{} = add i32 1, %{}", new_idx, old_idx));
    }

    // object
    pub fn add_class_declare(&mut self, class_id: usize, fields: &Vec<CGType>) {
        self.add_line_no_indent(format!("%class_{} = type {{ {} }}",
                                        class_id,
                                        join(fields, ',', CGType::user_type)));
    }

    pub fn get_field_addr(&mut self, struct_ptr: Val, t: CGType, idx: usize) -> Register {
        self.new_reg(format!("getelementptr {}, {}* {}, i32 0, i32 {}",
                             t.native_type(),
                             t.native_type(),
                             struct_ptr,
                             idx))
    }

    // function
    pub fn add_func_declare(&mut self, ret_type: CGType, func_name: &String, args: &Vec<CGType>) {
        let args_str = join(args, ',', CGType::user_type);
        self.add_line_no_indent(format!("declare {} @{}({})", ret_type, func_name, args_str));
    }

    pub fn add_func_begin(&mut self,
                          ret_type: CGType,
                          func_name: &String,
                          args: &Vec<CGType>)
                          -> Vec<(Register, CGType)> {
        let mut arg_regs: Vec<(Register, CGType)> = Vec::new();
        for arg_t in args {
            let reg = self.next_reg();
            arg_regs.push((reg, *arg_t));
        }

        let args_str = join(&arg_regs, ',', |(reg, arg_t)| format!("{} %{}", arg_t, reg));
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
        let args_str = join(args, ',', |(val, val_t)| format!("{} {}", val_t, val));
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
        let str_ptr = self.new_reg(format!("getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 \
                                            0",
                                           str_size + 1,
                                           str_size + 1,
                                           str_const));
        let str_t = CGType::new(RawType::TString);
        let struct_ptr = self.alloc_string();
        self.retain_string(Val::Reg(struct_ptr));
        let addr = self.get_field_addr(Val::Reg(struct_ptr), str_t, 1);
        self.add_raw_store(addr, format!("i8*"), Val::Reg(str_ptr));
        let addr = self.get_field_addr(Val::Reg(struct_ptr), str_t, 2);
        self.add_raw_store(addr, format!("i1"), Val::Int(1));
        struct_ptr
    }

    pub fn alloc_string(&mut self) -> Register {
        self.add_comment(format!("Allocating empty string"));
        let str_t = CGType::new(RawType::TString);
        let struct_ptr = self.add_malloc1(str_t.native_type());
        let reg = self.new_reg(format!("insertvalue {} undef, i32 0, 0", str_t.native_type()));
        let reg =
            self.new_reg(format!("insertvalue {} %{}, i8* null, 1", str_t.native_type(), reg));
        let reg =
            self.new_reg(format!("insertvalue {} %{}, i1 false, 2", str_t.native_type(), reg));
        self.add_raw_store(struct_ptr, str_t.native_type(), Val::Reg(reg));
        struct_ptr
    }

    pub fn retain_string(&mut self, struct_ptr: Val) {
        self.add_call(CGType::new(RawType::TVoid),
                      &format!("._retain_str"),
                      &vec![(struct_ptr, CGType::new(RawType::TString))]);
    }

    pub fn release_string(&mut self, struct_ptr: Val) {
        self.add_call(CGType::new(RawType::TVoid),
                      &format!("._release_str"),
                      &vec![(struct_ptr, CGType::new(RawType::TString))]);
    }

    pub fn new_arr(&mut self, arr_t: CGType, size: Val) -> Register {
        let struct_ptr = self.add_malloc1(arr_t.native_type());
        let arr_ptr = self.add_malloc(arr_t.t.user_type(), size);

        self.add_comment(format!("Constructing array"));
        let reg =
            self.new_reg(format!("insertvalue {} undef, i32 {}, 0", arr_t.native_type(), size));
        let reg = self.new_reg(format!("insertvalue {} %{}, {}* %{}, 1",
                                       arr_t.native_type(),
                                       reg,
                                       arr_t.t.user_type(),
                                       arr_ptr));
        self.add_comment(format!("Storing array"));
        self.add_raw_store(struct_ptr, arr_t.native_type(), Val::Reg(reg));

        if arr_t.t == RawType::TString {
            self.add_call(CGType::new(RawType::TVoid),
                          &format!("._init_str_arr"),
                          &vec![(Val::Reg(struct_ptr), arr_t)]);
        }

        struct_ptr
    }

    pub fn new_object(&mut self, t: CGType) -> Register {
        self.add_malloc1(t.native_type())
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
        let struct_val = self.add_raw_load(struct_ptr.to_reg(), t.native_type());
        self.add_comment(format!("Getting array pointer"));
        let elem0_ptr =
            self.new_reg(format!("extractvalue {} %{}, 1", t.native_type(), struct_val));
        let elem_ptr = self.new_reg(format!("getelementptr {}, {} %{}, i32 {}",
                                            t.t.user_type(),
                                            t.t.in_arr_type(),
                                            elem0_ptr,
                                            idx));
        (elem_ptr, CGType::new(t.t))
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

    fn add_raw_store(&mut self, addr_reg: Register, t: String, val: Val) {
        self.add_line(format!("store {} {}, {}* %{}", t, val, t, addr_reg));
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
    pub fn add_comment(&mut self, s: String) {
        self.add_line_no_indent(format!("; {}", s));
    }

    fn add_line(&mut self, s: String) {
        self.br_last_op = false;
        self.out.push(format!("\t{}", s));
    }

    fn add_line_no_indent(&mut self, s: String) {
        self.br_last_op = false;
        self.out.push(s);
    }

    pub fn add_empty_line(&mut self) {
        self.out.push(format!(""));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Val {
    Reg(Register),
    Int(i32),
    Null,
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
            Val::Null => write!(f, "null"),
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
    TRawPtr,
    TObject(usize),
    TNull,
}

impl fmt::Display for CGType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.user_type())
    }
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

    fn user_type(self) -> String {
        if self.is_arr {
            format!("{{ i32, {} }}*", self.t.in_arr_type())
        } else {
            format!("{}", self.t.user_type())
        }
    }

    fn native_type(self) -> String {
        if self.is_arr {
            format!("{{ i32, {} }}", self.t.in_arr_type())
        } else {
            format!("{}", self.t.native_type())
        }
    }
}

impl RawType {
    fn from(t: &Type) -> RawType {
        match *t {
            Type::TInt => RawType::TInt,
            Type::TBool => RawType::TBool,
            Type::TString => RawType::TString,
            Type::TVoid => RawType::TVoid,
            _ => unreachable!(),
        }
    }

    fn in_arr_type(self) -> String {
        format!("{}*", self.user_type())
    }

    fn user_type(self) -> String {
        match self {
            RawType::TString |
            RawType::TObject(_) => format!("{}*", self.native_type()),
            _ => format!("{}", self.native_type()),
        }
    }

    fn native_type(self) -> String {
        match self {
            RawType::TInt => format!("i32"),
            RawType::TBool => format!("i1"),
            RawType::TString => format!("{{ i32, i8*, i1 }}"), // ref_count, char*, is_const
            RawType::TVoid => format!("void"),
            RawType::TRawPtr => format!("i8*"),
            RawType::TObject(x) => format!("%class_{}", x),
            RawType::TNull => panic!("null is not a valid type"),
        }
    }
}
