use std::collections::HashMap;

use ast::{Type, Ident};

use code_generation::code_generator::*;

#[derive(Debug)]
pub struct Context {
    vars: HashMap<Ident, (Register, CGType)>,
    func_types: HashMap<Ident, (Vec<CGType>, CGType)>,
    string_lits: HashMap<String, StrConstant>,
    pub ret_type: CGType,

    classes: HashMap<usize, ClassData>,
    class_ids: HashMap<Ident, usize>,

    string_tmps: Vec<Val>,
    local_string_tmps: Vec<Val>,
    string_vars: Vec<Register>,
    local_string_vars: Vec<Register>,

    pub cg: CodeGenerator,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub id: usize,
    pub super_id: Option<usize>,
    pub ident: Ident,
    fields: Vec<CGType>,
    field_ids: HashMap<Ident, usize>,
}

impl ClassData {
    pub fn new(id: usize, ident: &Ident) -> ClassData {
        ClassData {
            id: id,
            super_id: None,
            ident: ident.clone(),
            fields: Vec::new(),
            field_ids: HashMap::new(),
        }
    }

    pub fn set_super(&mut self, id: usize) {
        self.super_id = Some(id);
    }

    pub fn get_super(&self) -> usize {
        self.super_id.unwrap()
    }

    pub fn has_field(&self, ident: &Ident) -> bool {
        self.field_ids.get(ident).is_some()
    }

    pub fn get_fields(&self) -> Vec<Ident> {
        self.field_ids.keys().map(|k| k.clone()).collect()
    }

    pub fn add_field(&mut self, ident: &Ident, t: CGType) {
        let id = self.fields.len();
        self.field_ids.insert(ident.clone(), id);
        self.fields.push(t);
    }

    pub fn get_field_type(&self, ident: &Ident) -> CGType {
        self.fields[self.get_field_idx(ident)]
    }

    pub fn get_field_id(&self, ident: &Ident) -> usize {
        let mut res = self.get_field_idx(ident);
        if self.super_id.is_some() {
            res += 1;
        }
        res
    }

    fn get_field_idx(&self, ident: &Ident) -> usize {
        *self.field_ids.get(ident).unwrap()
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            vars: HashMap::new(),
            func_types: HashMap::new(),
            string_lits: HashMap::new(),
            ret_type: CGType::new(RawType::TVoid),

            classes: HashMap::new(),
            class_ids: HashMap::new(),

            string_tmps: Vec::new(),
            local_string_tmps: Vec::new(),
            string_vars: Vec::new(),
            local_string_vars: Vec::new(),

            cg: CodeGenerator::new(),
        }
    }

    pub fn in_new_scope<F, R>(&mut self, f: F) -> R
        where F: Fn(&mut Context) -> R
    {
        let old_vars = self.vars.clone();
        let old_string_tmps = self.string_tmps.clone();
        let old_local_string_tmps = self.local_string_tmps.clone();
        let old_string_vars = self.string_vars.clone();
        let old_local_string_vars = self.local_string_vars.clone();

        self.local_string_tmps.clear();
        self.local_string_vars.clear();
        let res = f(self);

        self.local_string_vars = old_local_string_vars;
        self.string_vars = old_string_vars;
        self.local_string_tmps = old_local_string_tmps;
        self.string_tmps = old_string_tmps;
        self.vars = old_vars;
        res
    }

    pub fn get_arg_types(&self, ident: &Ident) -> Vec<CGType> {
        self.func_types.get(ident).unwrap().0.clone()
    }

    pub fn get_ret_type(&self, ident: &Ident) -> CGType {
        self.func_types.get(ident).unwrap().1
    }

    pub fn get_str_const(&self, s: &String) -> StrConstant {
        *self.string_lits.get(s).unwrap()
    }

    pub fn set_str_const(&mut self, s: String, reg: StrConstant) {
        self.string_lits.insert(s, reg);
    }

    pub fn get_var(&self, ident: &Ident) -> (Register, CGType) {
        *self.vars.get(ident).unwrap()
    }

    pub fn set_var(&mut self, ident: Ident, addr_reg: Register, t: CGType) {
        self.vars.insert(ident.clone(), (addr_reg, t));
        if t == CGType::new(RawType::TString) {
            self.string_vars.push(addr_reg);
            self.local_string_vars.push(addr_reg);
        }
    }

    pub fn add_func(&mut self, ident: &Ident, arg_types: Vec<CGType>, ret_type: CGType) {
        self.func_types.insert(ident.clone(), (arg_types, ret_type));
    }

    // string reference counting
    pub fn add_string_tmp(&mut self, reg: Val) {
        self.string_tmps.push(reg);
        self.local_string_tmps.push(reg);
    }

    pub fn release_local_strings(&mut self) {
        let strs = self.local_string_tmps.clone();
        self.release_string_tmps(strs);
        let strs = self.local_string_vars.clone();
        self.release_string_vars(strs);
    }

    pub fn release_all_strings(&mut self) {
        let strs = self.string_tmps.clone();
        self.release_string_tmps(strs);
        let strs = self.string_vars.clone();
        self.release_string_vars(strs);
    }

    fn release_string_tmps(&mut self, regs: Vec<Val>) {
        for reg in regs {
            self.cg.release_string(reg);
        }
    }

    fn release_string_vars(&mut self, var_addrs: Vec<Register>) {
        let str_t = CGType::new(RawType::TString);
        for var_addr in var_addrs {
            let val = self.cg.add_load(var_addr, str_t);
            self.cg.release_string(Val::Reg(val));
        }
    }

    // class
    pub fn add_class(&mut self, id: usize, cdata: ClassData) {
        let cname = cdata.ident.clone();
        self.cg.add_comment(format!("class {}", cname));
        if let Some(super_id) = cdata.super_id {
            self.cg.add_subclass_declare(id, super_id, &cdata.fields);
        } else {
            self.cg.add_class_declare(id, &cdata.fields);
        }
        self.classes.insert(id, cdata);
        self.class_ids.insert(cname, id);
    }

    pub fn get_class_data(&self, id: usize) -> &ClassData {
        self.classes.get(&id).unwrap()
    }

    pub fn to_cgtype(&self, t: &Type) -> CGType {
        match *t {
            Type::TObject(ref cname) => {
                CGType::new(RawType::TObject(*self.class_ids.get(cname).unwrap()))
            }
            Type::TArray(ref elem_t) => CGType::new_arr(self.to_cgtype(elem_t).t),
            _ => CGType::from(t),
        }
    }
}
