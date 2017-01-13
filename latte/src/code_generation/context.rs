use std::collections::HashMap;

use ast::Ident;

use code_generation::code_generator::*;

#[derive(Debug)]
pub struct Context {
    pub vars: HashMap<Ident, (Register, CGType)>,
    pub func_ret_types: HashMap<Ident, CGType>,
    pub string_lits: HashMap<String, StrConstant>,

    pub string_tmps: Vec<Val>,
    pub local_string_tmps: Vec<Val>,
    pub string_vars: Vec<Register>,
    pub local_string_vars: Vec<Register>,

    pub cg: CodeGenerator,
}

impl Context {
    pub fn new(func_ret_types: HashMap<Ident, CGType>) -> Context {
        Context {
            vars: HashMap::new(),
            func_ret_types: func_ret_types,
            string_lits: HashMap::new(),

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

    pub fn get_ret_type(&self, ident: &Ident) -> CGType {
        *self.func_ret_types.get(ident).unwrap()
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
}
