use ast::Program;

mod class_hierarchy_check;
mod result;
mod return_check;
mod return_error;
mod type_check;
mod type_context;
mod type_error;

pub mod collect_string_lit;
pub mod has_return;

pub fn run(p: &Program) -> result::Result {
    class_hierarchy_check::run(p).map_err(|e| result::Error::Class(e))?;
    type_check::run(p).map_err(|e| result::Error::Type(e))?;
    Ok(())
}

pub fn check_returns(p: &Program) -> result::Result {
    return_check::run(p).map_err(|e| result::Error::Return(e))
}
