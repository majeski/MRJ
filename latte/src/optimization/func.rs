use ast::*;

use optimization::optimize::*;

impl Optimize for Func {
    fn optimize(self) -> Func {
        Func {
            ident: self.ident,
            args: self.args,
            ret_type: self.ret_type,
            body: self.body.optimize(),
        }
    }
}
