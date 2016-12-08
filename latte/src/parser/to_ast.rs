pub trait ToAst<T> {
    fn to_ast(&self) -> T;
}

impl<T, E> ToAst<T> for *mut E
    where E: ToAst<T>
{
    fn to_ast(&self) -> T {
        if self.is_null() {
            panic!("unexpected NULL");
        }
        unsafe { (**self).to_ast() }
    }
}
