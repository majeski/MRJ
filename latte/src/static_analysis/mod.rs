use ast::Program;

mod class_hierarchy_check;
mod result;
mod return_check;
mod return_error;
mod type_check;
mod type_context;
mod type_error;

pub fn run(p: &Program) -> result::Result {
    class_hierarchy_check::run(p).map_err(|e| result::Error::Class(e))?;
    type_check::run(p).map_err(|e| result::Error::Type(e))?;
    return_check::run(p).map_err(|e| result::Error::Return(e))?;
    Ok(())
}
