use std::collections::HashMap;

use ast::{Ident, Type};

#[derive(Debug)]
pub struct TypeContext {
    // identifier -> (type, current_scope)
    idents: HashMap<Ident, (Type, bool)>,
    ret_type: Type,
}

impl TypeContext {
    pub fn new() -> TypeContext {
        TypeContext {
            idents: HashMap::new(),
            ret_type: Type::TVoid,
        }
    }

    pub fn in_new_scope<F, T>(&self, f: F) -> T
        where F: Fn(TypeContext) -> T
    {
        let ctx = self._new_scope();
        f(ctx)
    }

    pub fn in_function_scope<F, T>(&self, ret_type: &Type, f: F) -> T
        where F: Fn(TypeContext) -> T
    {
        let mut ctx = self._new_scope();
        ctx.ret_type = ret_type.clone();
        f(ctx)
    }

    fn _new_scope(&self) -> TypeContext {
        let mut idents = self.idents.clone();
        idents.iter_mut().map(|(_, ref mut e)| e.1 = false).collect::<Vec<()>>();
        let ret_type = self.ret_type.clone();
        TypeContext {
            idents: idents,
            ret_type: ret_type,
        }
    }

    pub fn get(&self, ident: &Ident) -> Option<Type> {
        self.idents.get(ident).map(|e| e.0.clone())
    }

    pub fn is_local(&self, ident: &Ident) -> bool {
        self.idents.get(ident).map(|e| e.1).unwrap_or(false)
    }

    pub fn set(&mut self, ident: &Ident, t: &Type) {
        self.idents.insert(ident.clone(), (t.clone(), true));
    }

    pub fn get_ret_type(&self) -> Type {
        self.ret_type.clone()
    }
}
