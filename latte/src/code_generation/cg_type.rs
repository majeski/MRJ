use std::fmt;

use ast::Type;

pub type ClassId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CGType {
    is_arr: bool,
    t: RawType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawType {
    TInt,
    TBool,
    TVoid,
    TString,
    TRawPtr,
    TObject(ClassId),
    TNull,
}

impl fmt::Display for CGType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.user_type())
    }
}

impl CGType {
    pub fn int_t() -> CGType {
        Self::new(RawType::TInt)
    }

    pub fn bool_t() -> CGType {
        Self::new(RawType::TBool)
    }

    pub fn void_t() -> CGType {
        Self::new(RawType::TVoid)
    }

    pub fn str_t() -> CGType {
        Self::new(RawType::TString)
    }

    pub fn ptr_t() -> CGType {
        Self::new(RawType::TRawPtr)
    }

    pub fn null_t() -> CGType {
        Self::new(RawType::TNull)
    }

    pub fn obj_t(id: ClassId) -> CGType {
        Self::new(RawType::TObject(id))
    }

    pub fn arr_t(t: RawType) -> CGType {
        CGType {
            is_arr: true,
            t: t,
        }
    }

    fn new(t: RawType) -> CGType {
        CGType {
            is_arr: false,
            t: t,
        }
    }

    pub fn as_raw(self) -> RawType {
        self.t
    }

    pub fn arr_elem_t(mut self) -> CGType {
        self.is_arr = false;
        self
    }

    pub fn get_id(self) -> ClassId {
        match self.t {
            RawType::TObject(id) => id,
            _ => panic!(),
        }
    }

    pub fn is_arr(self) -> bool {
        self.is_arr
    }

    pub fn is_obj(self) -> bool {
        if let RawType::TObject(_) = self.t {
            true
        } else {
            false
        }
    }

    pub fn from(t: &Type) -> CGType {
        match *t {
            Type::TArray(ref t) => Self::arr_t(RawType::from(t)),
            _ => Self::new(RawType::from(t)),
        }
    }

    pub fn user_type(&self) -> String {
        if self.is_arr {
            format!("{{ i32, {} }}*", self.t.in_arr_type())
        } else {
            format!("{}", self.t.user_type())
        }
    }

    pub fn native_type(&self) -> String {
        if self.is_arr {
            format!("{{ i32, {} }}", self.t.in_arr_type())
        } else {
            format!("{}", self.t.native_type())
        }
    }
}

impl RawType {
    fn from(t: &Type) -> RawType {
        match *t {
            Type::TInt => RawType::TInt,
            Type::TBool => RawType::TBool,
            Type::TString => RawType::TString,
            Type::TVoid => RawType::TVoid,
            _ => unreachable!(),
        }
    }

    pub fn in_arr_type(self) -> String {
        format!("{}*", self.user_type())
    }

    pub fn user_type(&self) -> String {
        match *self {
            RawType::TString |
            RawType::TObject(_) => format!("{}*", self.native_type()),
            _ => format!("{}", self.native_type()),
        }
    }

    pub fn native_type(&self) -> String {
        match *self {
            RawType::TInt => format!("i32"),
            RawType::TBool => format!("i1"),
            RawType::TString => format!("%string_t"), // ref_count, char*, is_const
            RawType::TVoid => format!("void"),
            RawType::TRawPtr => format!("i8*"),
            RawType::TObject(x) => format!("%class_{}", x),
            RawType::TNull => panic!("null is not a valid type"),
        }
    }
}
