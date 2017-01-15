use ast::*;

use optimization::optimize::*;

impl Optimize for Def {
    fn optimize(self) -> Def {
        match self {
            Def::DFunc(f) => Def::DFunc(f.optimize()),
            Def::DClass(c) => Def::DClass(c), // TODO
        }
    }
}
