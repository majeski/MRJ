use std::collections::HashMap;

use ast::Ident;

use code_generation::cg_type::*;
use code_generation::code_generator::*;
use code_generation::vtable::*;

#[derive(Debug, Clone)]
pub struct ClassData {
    pub id: usize,
    pub super_id: Option<usize>,
    pub ident: Ident,
    pub fields: Vec<CGType>,
    field_ids: HashMap<Ident, usize>,
    pub vtable: VTable,
    pub vtable_addr: VTableConstant,
}

impl ClassData {
    pub fn new(id: usize, ident: &Ident) -> ClassData {
        ClassData {
            id: id,
            super_id: None,
            ident: ident.clone(),
            fields: Vec::new(),
            field_ids: HashMap::new(),
            vtable: VTable::new(),
            vtable_addr: VTableConstant(0),
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
        self.get_field_idx(ident) + 1
    }

    fn get_field_idx(&self, ident: &Ident) -> usize {
        *self.field_ids.get(ident).unwrap()
    }
}
