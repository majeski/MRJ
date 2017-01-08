use ast::{Def, Program};

mod def;
mod expr;
mod func;
mod optimize;
mod stmt;

use self::optimize::*;

pub fn run(p: Program) -> Program {
    Program(p.0.into_iter().map(Def::optimize).collect())
}
