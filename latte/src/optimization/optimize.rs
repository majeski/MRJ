pub trait Optimize {
    fn optimize(self) -> Self;
}

impl<T: Sized> Optimize for Box<T>
    where T: Optimize
{
    fn optimize(self) -> Box<T> {
        Box::new((*self).optimize())
    }
}

