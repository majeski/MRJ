use std::collections::HashMap;

use ast::Ident;

use code_generation::code_generator::*;

#[derive(Debug)]
pub struct Context {
    pub vars: HashMap<Ident, (Register, CGType)>,
    pub func_ret_types: HashMap<Ident, CGType>,
    pub string_lits: HashMap<String, StrConstant>,
    pub cg: CodeGenerator,
}

impl Context {
    pub fn new(func_ret_types: HashMap<Ident, CGType>) -> Context {
        Context {
            vars: HashMap::new(),
            func_ret_types: func_ret_types,
            string_lits: HashMap::new(),
            cg: CodeGenerator::new(),
        }
    }

    pub fn in_new_scope<F, R>(&mut self, f: F) -> R
        where F: Fn(&mut Context) -> R
    {
        let old_vars = self.vars.clone();
        let res = f(self);
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
    }
}
