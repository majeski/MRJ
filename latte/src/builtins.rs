use ast::{BuiltinFunc, Ident, Type};

pub fn get_builtin_functions() -> Vec<BuiltinFunc> {
    vec![get_print_int(), get_print_string(), get_error(), get_read_int(), get_read_string()]
}

fn get_print_int() -> BuiltinFunc {
    BuiltinFunc {
        ident: Ident(String::from("printInt")),
        args: vec![Type::TInt],
        ret_type: Type::TVoid,
    }
}

fn get_print_string() -> BuiltinFunc {
    BuiltinFunc {
        ident: Ident(String::from("printString")),
        args: vec![Type::TString],
        ret_type: Type::TVoid,
    }
}

fn get_error() -> BuiltinFunc {
    BuiltinFunc {
        ident: Ident(String::from("error")),
        args: vec![],
        ret_type: Type::TVoid,
    }
}

fn get_read_int() -> BuiltinFunc {
    BuiltinFunc {
        ident: Ident(String::from("readInt")),
        args: vec![],
        ret_type: Type::TInt,
    }
}

fn get_read_string() -> BuiltinFunc {
    BuiltinFunc {
        ident: Ident(String::from("readString")),
        args: vec![],
        ret_type: Type::TString,
    }
}
