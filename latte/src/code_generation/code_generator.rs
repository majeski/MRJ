use std::fmt;

use ast::Operator;

use code_generation::cg_type::*;
use code_generation::utils::*;

#[derive(Debug)]
pub struct CodeGenerator {
    out: Vec<String>,
    last_reg: i32,
    last_label: i32,
    last_str_const: i32,
    current_label: Label,
}

impl CodeGenerator {
    pub fn new() -> CodeGenerator {
        let mut cg = CodeGenerator {
            out: Vec::new(),
            last_reg: 0,
            last_label: 0,
            last_str_const: 0,
            current_label: Label(-1),
        };

        cg.add_line_no_indent(format!("%string_t = type {{ i32, i8*, i1 }}"));
        cg.add_empty_line();

        cg.add_comment(format!("internal functions"));
        for (ret_type, name, args) in Self::internal_functions() {
            cg.add_func_declare(ret_type, &name, &args);
        }
        cg.add_empty_line();
        cg
    }

    fn internal_functions() -> Vec<(CGType, String, Vec<CGType>)> {
        vec![
            (CGType::str_t(), format!("._concatenate"), vec![CGType::str_t(), CGType::str_t()]),
            (CGType::str_t(), format!("._alloc_str"), vec![]),
            (CGType::void_t(), format!("._retain_str"), vec![CGType::str_t()]),
            (CGType::void_t(), format!("._release_str"), vec![CGType::str_t()]),
            (CGType::void_t(), format!("._init_str_arr"), vec![CGType::arr_t(RawType::TString)]),
            (CGType::ptr_t(), format!("malloc"), vec![CGType::int_t()]),
        ]
    }

    pub fn reset(&mut self) {
        self.last_reg = 0;
        self.last_label = 0;
    }

    pub fn get_out(&self) -> &Vec<String> {
        &self.out
    }

    pub fn add_phi(&mut self, t: CGType, op1: (Val, Label), op2: (Val, Label)) -> Val {
        self.new_reg(format!("phi {} [{}, %{}], [{}, %{}]", t, op1.0, op1.1, op2.0, op2.1))
    }

    pub fn add_int_op(&mut self, lhs: Val, op: Operator, rhs: Val) -> Val {
        self.add_op(CGType::int_t(), lhs, op, rhs)
    }

    pub fn add_op(&mut self, t: CGType, lhs: Val, op: Operator, rhs: Val) -> Val {
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

    pub fn add_neg(&mut self, val: Val) -> Val {
        self.new_reg(format!("sub i32 0, {}", val))
    }

    pub fn add_not(&mut self, val: Val) -> Val {
        self.new_reg(format!("add i1 1, {}", val))
    }

    pub fn concatenate_str(&mut self, lhs: Val, rhs: Val) -> Val {
        let str_t = CGType::str_t();
        self.new_reg(format!("call {0} @._concatenate({0} {1}, {0} {2})", str_t, lhs, rhs))
    }

    pub fn add_loop_step(&mut self, new_idx: Register, old_idx: Val) {
        self.add_line(format!("%{} = add i32 1, {}", new_idx, old_idx));
    }

    // object
    pub fn add_class_declare(&mut self, class_id: ClassId, fields: &Vec<CGType>) {
        let vtable_t = format!("i32 (...)**");
        let fields_str = if fields.is_empty() {
            vtable_t
        } else {
            format!("{}, {}", vtable_t, join(fields, ',', CGType::user_type))
        };
        self.add_line_no_indent(format!("%class_{} = type {{ {} }}", class_id, fields_str));
    }

    pub fn add_subclass_declare(&mut self,
                                class_id: ClassId,
                                super_id: ClassId,
                                fields: &Vec<CGType>) {
        let fields_str = if fields.is_empty() {
            format!("")
        } else {
            format!(", {}", join(fields, ',', CGType::user_type))
        };
        self.add_line_no_indent(format!("%class_{} = type {{ %class_{}{} }}",
                                        class_id,
                                        super_id,
                                        fields_str));
    }

    pub fn add_vtable_declare(&mut self,
                              class_id: ClassId,
                              size: usize,
                              arr: String)
                              -> VTableConstant {
        let reg = VTableConstant(class_id);
        self.add_line_no_indent(format!("@{} = private unnamed_addr constant [{} x i8*] {}",
                                        reg,
                                        size,
                                        arr));
        reg
    }

    pub fn store_vtable(&mut self, obj_addr: Val, class_id: ClassId, size: usize) {
        let arr_t = format!("[{} x i8*]", size);
        let vtable_t = format!("i32 (...)**");
        let dst_addr =
            self.new_reg(format!("bitcast %class_{}* {} to {}*", class_id, obj_addr, vtable_t));

        let addr = self.new_reg(format!("getelementptr {0}, {0}* @.vtable_{1}, i64 0, i64 0",
                                        arr_t,
                                        class_id));
        let val = self.new_reg(format!("bitcast i8** {} to {}", addr, vtable_t));
        self.add_raw_store(dst_addr, vtable_t, val);
    }

    pub fn load_vtable_entry(&mut self,
                             obj_addr: Val,
                             class_id: ClassId,
                             ftype: String,
                             idx: usize)
                             -> Val {
        let vtable_addr =
            self.new_reg(format!("bitcast %class_{}* {} to {}**", class_id, obj_addr, ftype));
        let vtable_reg = self.add_raw_load(vtable_addr, format!("{}*", ftype));
        let faddr = self.new_reg(format!("getelementptr {0}, {0}* {1}, i64 {2}",
                                         ftype,
                                         vtable_reg,
                                         idx));
        let f = self.add_raw_load(faddr, ftype);
        f
    }


    pub fn bitcast_object(&mut self, addr: Val, from: CGType, to: CGType) -> Val {
        self.new_reg(format!("bitcast {} {} to {}",
                             from.user_type(),
                             addr,
                             to.user_type()))
    }

    pub fn get_field_addr(&mut self, struct_ptr: Val, t: CGType, idx: usize) -> Val {
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
                          -> Vec<(Val, CGType)> {
        let mut arg_regs: Vec<(Register, CGType)> = Vec::new();
        for arg_t in args {
            let reg = self.next_reg();
            arg_regs.push((reg, *arg_t));
        }

        let args_str = join(&arg_regs,
                            ',',
                            |&(reg, arg_t)| format!("{} %{}", arg_t, reg));
        self.add_line_no_indent(format!("define {} @{}({}) {}",
                                        ret_type,
                                        func_name,
                                        args_str,
                                        '{'));

        let mut arg_addrs: Vec<(Val, CGType)> = Vec::new();
        for arg in &arg_regs {
            let addr_reg = self.add_alloca(arg.1);
            self.add_store(addr_reg, arg.1, Val::Reg(arg.0));
            if arg.1 == CGType::str_t() {
                self.retain_string(Val::Reg(arg.0));
            }
            arg_addrs.push((addr_reg, arg.1));
        }
        arg_addrs
    }

    pub fn add_func_end(&mut self, ret_type: CGType) {
        if ret_type == CGType::void_t() {
            self.add_line(format!("ret void"));
        }
        self.add_line_no_indent(format!("{}", '}'));
        self.add_line_no_indent(format!(""));
    }

    pub fn add_call(&mut self,
                    ret_type: CGType,
                    func_name: String,
                    args: &Vec<(Val, CGType)>)
                    -> Val {
        let args_str = join(args, ',', |&(val, val_t)| format!("{} {}", val_t, val));
        let call_str = format!("call {} {}({})", ret_type, func_name, args_str);
        if ret_type == CGType::void_t() {
            self.add_line(call_str);
            Val::Reg(self.dummy_reg())
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

    pub fn add_str_load(&mut self, str_size: usize, str_const: StrConstant) -> Val {
        let str_ptr = self.new_reg(format!("getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 \
                                            0",
                                           str_size + 1,
                                           str_size + 1,
                                           str_const));
        let str_t = CGType::str_t();
        let struct_ptr = self.alloc_string();
        self.retain_string(struct_ptr);
        let addr = self.get_field_addr(struct_ptr, str_t, 2);
        self.add_store(addr, CGType::bool_t(), Val::Int(1)); // is_const = 1
        let addr = self.get_field_addr(struct_ptr, str_t, 1);
        self.add_store(addr, CGType::ptr_t(), str_ptr);
        struct_ptr
    }

    pub fn alloc_string(&mut self) -> Val {
        self.add_call(CGType::str_t(), format!("@._alloc_str"), &vec![])
    }

    pub fn retain_string(&mut self, struct_ptr: Val) {
        self.add_call(CGType::void_t(),
                      format!("@._retain_str"),
                      &vec![(struct_ptr, CGType::str_t())]);
    }

    pub fn release_string(&mut self, struct_ptr: Val) {
        self.add_call(CGType::void_t(),
                      format!("@._release_str"),
                      &vec![(struct_ptr, CGType::str_t())]);
    }

    pub fn new_arr(&mut self, arr_t: CGType, size: Val) -> Val {
        let struct_ptr = self.add_malloc1(arr_t.native_type());
        let arr_ptr = self.add_malloc(arr_t.arr_elem_t().user_type(), size);

        let reg =
            self.new_reg(format!("insertvalue {} undef, i32 {}, 0", arr_t.native_type(), size));
        let reg = self.new_reg(format!("insertvalue {} {}, {}* {}, 1",
                                       arr_t.native_type(),
                                       reg,
                                       arr_t.arr_elem_t().user_type(),
                                       arr_ptr));
        self.add_raw_store(struct_ptr, arr_t.native_type(), reg);

        if arr_t.arr_elem_t() == CGType::str_t() {
            self.add_call(CGType::void_t(),
                          format!("@._init_str_arr"),
                          &vec![(struct_ptr, arr_t)]);
        }

        struct_ptr
    }

    pub fn new_object(&mut self, t: CGType) -> Val {
        self.add_malloc1(t.native_type())
    }

    fn add_malloc1(&mut self, t: String) -> Val {
        self.add_malloc(t, Val::Int(1))
    }

    fn add_malloc(&mut self, t: String, size: Val) -> Val {
        let size_of = self.get_sizeof(t.clone(), size);
        let void_addr = self.new_reg(format!("call i8* @malloc(i32 {})", size_of));
        let cast_addr = self.new_reg(format!("bitcast i8* {} to {}*", void_addr, t));
        cast_addr
    }

    fn get_sizeof(&mut self, t: String, size: Val) -> Val {
        let size_of = self.new_reg(format!("getelementptr {}, {}* null, i32 {}", t, t, size));
        let res = self.new_reg(format!("ptrtoint {}* {} to i32", t, size_of));
        res
    }

    pub fn get_nth_arr_elem(&mut self, struct_ptr: Val, t: CGType, idx: Val) -> (Val, CGType) {
        let struct_val = self.add_raw_load(struct_ptr, t.native_type());
        let elem0_ptr = self.new_reg(format!("extractvalue {} {}, 1", t.native_type(), struct_val));
        let elem_ptr = self.new_reg(format!("getelementptr {}, {} {}, i32 {}",
                                            t.arr_elem_t().user_type(),
                                            t.as_raw().in_arr_type(),
                                            elem0_ptr,
                                            idx));
        (elem_ptr, t.arr_elem_t())
    }

    pub fn add_alloca(&mut self, t: CGType) -> Val {
        self.new_reg(format!("alloca {}", t))
    }

    pub fn add_load(&mut self, addr_reg: Val, t: CGType) -> Val {
        self.add_raw_load(addr_reg, format!("{}", t))
    }

    fn add_raw_load(&mut self, addr_reg: Val, t: String) -> Val {
        self.new_reg(format!("load {}, {}* {}", t, t, addr_reg))
    }

    pub fn add_store(&mut self, addr_reg: Val, t: CGType, val: Val) {
        self.add_raw_store(addr_reg, format!("{}", t), val);
    }

    fn add_raw_store(&mut self, addr_reg: Val, t: String, val: Val) {
        self.add_line(format!("store {} {}, {}* {}", t, val, t, addr_reg));
    }

    // generating registers
    fn new_reg(&mut self, rhs: String) -> Val {
        let reg = self.next_reg();
        self.add_line(format!("%{} = {}", reg, rhs));
        Val::Reg(reg)
    }

    pub fn next_reg(&mut self) -> Register {
        self.last_reg += 1;
        Register(self.last_reg)
    }

    pub fn dummy_reg(&self) -> Register {
        Register(-100)
    }

    // labels & brs
    pub fn add_ret_void(&mut self) {
        self.add_line(format!("ret void"));
    }

    pub fn add_ret(&mut self, t: CGType, val: Val) {
        self.add_line(format!("ret {} {}", t, val));
    }

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
        self.add_line(format!("br i1 {}, label %{}, label %{}", cond, if_true, if_false));
    }

    pub fn add_jump(&mut self, l: Label) {
        self.add_line(format!("br label %{}", l));
    }

    // core functions
    pub fn add_comment(&mut self, s: String) {
        self.add_line_no_indent(format!("; {}", s));
    }

    fn add_line(&mut self, s: String) {
        self.out.push(format!("\t{}", s));
    }

    fn add_line_no_indent(&mut self, s: String) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(i32);

#[derive(Debug, Clone, Copy)]
pub struct StrConstant(i32);

#[derive(Debug, Clone, Copy)]
pub struct VTableConstant(pub ClassId);

#[derive(Debug, Clone, Copy)]
pub struct Label(i32);

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Val::Reg(ref r) => write!(f, "%{}", r),
            Val::Int(x) => write!(f, "{}", x),
            Val::Null => write!(f, "null"),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r_{}", self.0)
    }
}

impl fmt::Display for StrConstant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ".str_const_{}", self.0)
    }
}

impl fmt::Display for VTableConstant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ".vtable_{}", self.0)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "label_{}", self.0)
    }
}
