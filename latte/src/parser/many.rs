extern crate libc;

use libc::*;

#[repr(C)]
pub struct many_t {
    next: *mut many_t,
    elem: *mut c_void,
}

impl many_t {
    pub fn to_vec<ElemDstT, ElemSrcT, F>(mut many: *mut many_t, convert: F) -> Vec<ElemDstT>
        where F: Fn(&ElemSrcT) -> ElemDstT
    {
        let mut v: Vec<ElemDstT> = Vec::new();
        unsafe {
            while !many.is_null() {
                let ref elem = *((*many).elem as *mut ElemSrcT);
                v.push(convert(elem));
                many = (*many).next;
            }
        }
        v
    }
}
