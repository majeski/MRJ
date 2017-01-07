use ast::{FieldGet, Ident};

pub fn get_ident(field_get: &FieldGet) -> &Ident {
    // TODO
    &field_get.ident
}
