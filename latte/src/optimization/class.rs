use ast::*;

use optimization::optimize::*;

impl Optimize for Class {
    fn optimize(self) -> Class {
        Class {
            name: self.name,
            superclass: self.superclass,
            vars: self.vars,
            methods: self.methods.into_iter().map(Func::optimize).collect(),
        }
    }
}
