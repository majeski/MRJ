extern crate libc;

use libc::*;
use parser::to_ast::TAResult;

#[repr(C)]
pub struct many_t {
    next: *mut many_t,
    elem: *mut c_void,
}

impl many_t {
    pub fn to_vec<ElemDstT, ElemSrcT, F>(mut many: *mut many_t,
                                         convert: F)
                                         -> TAResult<Vec<ElemDstT>>
        where F: Fn(&ElemSrcT) -> TAResult<ElemDstT>
    {
        let mut v: Vec<ElemDstT> = Vec::new();
        unsafe {
            while !many.is_null() {
                let ref elem = *((*many).elem as *mut ElemSrcT);
                let converted = convert(elem)?;
                v.push(converted);
                many = (*many).next;
            }
        }
        Ok(v)
    }
}
