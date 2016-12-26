use std::collections::HashMap;

use ast::{Ident, Type};

type IdentsMap<T> = HashMap<Ident, T>;

#[derive(Debug)]
pub struct TypeContext {
    idents: IdentsMap<(Type, bool)>,
    class_data: IdentsMap<ClassData>,
    ret_type: Type,
}

#[derive(Debug, Clone)]
struct ClassData {
    name: Ident,
    superclass: Option<Ident>,
    fields: IdentsMap<Type>,
}

impl TypeContext {
    pub fn new() -> TypeContext {
        TypeContext {
            idents: HashMap::new(),
            class_data: HashMap::new(),
            ret_type: Type::TVoid,
        }
    }

    // execute in modified context

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

    pub fn in_class_scope<F, T>(&self, class_name: &Ident, retain_outer_scope: bool, f: F) -> T
        where F: Fn(TypeContext) -> T
    {
        let mut ctx = self._new_scope();
        if !retain_outer_scope {
            ctx.idents.clear();
        }
        for (ident, t) in self.get_fields(class_name) {
            ctx.set_type(&ident, &t);
        }
        f(ctx)
    }

    fn _new_scope(&self) -> TypeContext {
        let mut idents = self.idents.clone();
        idents.iter_mut().map(|(_, ref mut e)| e.1 = false).collect::<Vec<()>>();
        TypeContext {
            idents: idents,
            class_data: self.class_data.clone(),
            ret_type: self.ret_type.clone(),
        }
    }

    // classes

    pub fn add_class(&mut self,
                     name: &Ident,
                     superclass: &Option<Ident>,
                     fields: IdentsMap<Type>) {
        self.class_data.insert(name.clone(),
                               ClassData {
                                   name: name.clone(),
                                   superclass: superclass.clone(),
                                   fields: fields,
                               });
    }

    pub fn class_exists(&self, class_name: &Ident) -> bool {
        self.class_data.contains_key(class_name)
    }

    pub fn get_field_type(&self, class_name: &Ident, field: &Ident) -> Option<&Type> {
        let class_data = self.get_class_data(class_name);
        if let Some(ref t) = class_data.fields.get(field) {
            Some(t)
        } else if let Some(ref superclass) = class_data.superclass {
            self.get_field_type(superclass, field)
        } else {
            None
        }
    }

    pub fn is_subclass_of(&self, sub_name: &Ident, sup_name: &Ident) -> bool {
        let mut subclass = self.get_class_data(sub_name);
        while &subclass.name != sup_name && subclass.superclass.is_some() {
            subclass = self.get_class_data(&subclass.superclass.clone().unwrap());
        }
        &subclass.name == sup_name
    }

    fn get_fields(&self, class_name: &Ident) -> IdentsMap<Type> {
        let mut fields: IdentsMap<Type> = HashMap::new();
        self.do_get_fields(class_name, &mut fields);
        fields
    }

    fn do_get_fields(&self, class_name: &Ident, mut result: &mut IdentsMap<Type>) {
        let class_data = self.get_class_data(class_name);
        result.extend(class_data.fields.clone());
        if let Some(ref superclass) = class_data.superclass {
            self.do_get_fields(superclass, &mut result);
        }
    }

    fn get_class_data(&self, ident: &Ident) -> &ClassData {
        self.class_data.get(ident).unwrap()
    }

    // identifiers

    pub fn get_type(&self, ident: &Ident) -> Option<&Type> {
        self.idents.get(ident).map(|e| &e.0)
    }

    pub fn set_type(&mut self, ident: &Ident, t: &Type) {
        self.idents.insert(ident.clone(), (t.clone(), true));
    }

    pub fn is_local(&self, ident: &Ident) -> bool {
        self.idents.get(ident).map(|e| e.1).unwrap_or(false)
    }

    pub fn get_ret_type(&self) -> &Type {
        &self.ret_type
    }
}
