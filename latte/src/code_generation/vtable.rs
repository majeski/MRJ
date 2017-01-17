use std::collections::HashMap;

use ast::Ident;

use code_generation::cg_type::*;
use code_generation::utils::*;

#[derive(Debug, Clone)]
pub struct VTable {
    pub fs: Vec<VTableEntry>,
    pub idxs: HashMap<Ident, usize>,
}

impl VTable {
    pub fn new() -> VTable {
        VTable {
            fs: Vec::new(),
            idxs: HashMap::new(),
        }
    }

    pub fn set_func(&mut self, ident: &Ident, f: VTableEntry) {
        let cur_idx = self.idxs.get(ident).map(usize::clone);
        match cur_idx {
            None => {
                let idx = self.fs.len();
                self.idxs.insert(ident.clone(), idx);
                self.fs.push(f);
            }
            Some(idx) => {
                self.fs[idx] = f;
            }
        }
    }

    pub fn size(&self) -> usize {
        self.fs.len()
    }

    pub fn to_i8_arr(&self) -> String {
        format!("[{}]", join(&self.fs, ',', VTableEntry::as_i8_ptr))
    }
}

#[derive(Debug, Clone)]
pub struct VTableEntry {
    pub real_ident: Ident,
    pub ret_type: CGType,
    pub arg_types: Vec<CGType>,
}

impl VTableEntry {
    pub fn new(real_ident: Ident, ret_type: CGType, arg_types: Vec<CGType>) -> VTableEntry {
        VTableEntry {
            real_ident: real_ident,
            ret_type: ret_type,
            arg_types: arg_types,
        }
    }

    fn as_i8_ptr(&self) -> String {
        format!("i8* bitcast ({} @{} to i8*)",
                self.as_ptr(),
                self.real_ident)
    }

    pub fn as_ptr(&self) -> String {
        let args = join(&self.arg_types, ',', CGType::user_type);
        format!("{} ({})*", self.ret_type, args)
    }
}
