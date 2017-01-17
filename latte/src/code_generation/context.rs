use std::collections::HashMap;

use ast::{Type, Ident};

use code_generation::cg_type::*;
use code_generation::class_data::*;
use code_generation::code_generator::*;

#[derive(Debug)]
pub struct Context {
    vars: HashMap<Ident, (Val, CGType)>,
    func_types: HashMap<Ident, (Vec<CGType>, CGType)>,
    string_lits: HashMap<String, StrConstant>,
    pub ret_type: CGType,
    pub class: Option<ClassId>,

    classes: HashMap<ClassId, ClassData>,
    class_ids: HashMap<Ident, ClassId>,

    string_tmps: Vec<Val>,
    local_string_tmps: Vec<Val>,
    string_vars: Vec<Val>,
    local_string_vars: Vec<Val>,

    pub cg: CodeGenerator,
}

impl Context {
    pub fn new() -> Context {
        Context {
            vars: HashMap::new(),
            func_types: HashMap::new(),
            string_lits: HashMap::new(),
            ret_type: CGType::void_t(),
            class: None,

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

    pub fn func_exists(&self, ident: &Ident) -> bool {
        self.func_types.get(ident).is_some()
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

    pub fn var_exists(&self, ident: &Ident) -> bool {
        self.vars.get(ident).is_some()
    }

    pub fn get_var(&self, ident: &Ident) -> (Val, CGType) {
        *self.vars.get(ident).unwrap()
    }

    pub fn set_var(&mut self, ident: Ident, addr_reg: Val, t: CGType) {
        self.vars.insert(ident.clone(), (addr_reg, t));
        if t == CGType::str_t() {
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
        self.cg.add_comment(format!("Releasing temporary variables"));
        for reg in regs {
            self.cg.release_string(reg);
        }
    }

    fn release_string_vars(&mut self, var_addrs: Vec<Val>) {
        self.cg.add_comment(format!("Releasing local variables"));
        for var_addr in var_addrs {
            let reg = self.cg.add_load(var_addr, CGType::str_t());
            self.cg.release_string(reg);
        }
    }

    // class
    pub fn add_class(&mut self, id: ClassId, cdata: ClassData) {
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

    pub fn get_class_id(&self, cname: &Ident) -> ClassId {
        *self.class_ids.get(cname).unwrap()
    }

    pub fn get_class_data(&self, id: ClassId) -> &ClassData {
        self.classes.get(&id).unwrap()
    }

    pub fn is_subclass_of(&self, mut id: ClassId, super_id: ClassId) -> bool {
        while self.get_class_data(id).super_id.is_some() && id != super_id {
            id = self.get_class_data(id).get_super();
        }
        id == super_id
    }

    pub fn to_cgtype(&self, t: &Type) -> CGType {
        match *t {
            Type::TObject(ref cname) => CGType::obj_t(*self.class_ids.get(cname).unwrap()),
            Type::TArray(ref elem_t) => CGType::arr_t(self.to_cgtype(elem_t).as_raw()),
            _ => CGType::from(t),
        }
    }
}
