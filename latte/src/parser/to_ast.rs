pub type TAResult<T> = Result<T, String>;

pub trait ToAst<T> {
    fn to_ast(&self) -> TAResult<T>;
}

impl<T, E> ToAst<T> for *mut E
    where E: ToAst<T>
{
    fn to_ast(&self) -> TAResult<T> {
        if self.is_null() {
            panic!("unexpected NULL");
        }
        unsafe { (**self).to_ast() }
    }
}
