use ast::Program;

mod class_hierarchy_check;
mod return_check;
mod return_error;
mod type_check;
mod type_context;
mod type_error;

mod result;

pub fn analyse(p: &Program) -> result::Result {
    class_hierarchy_check::run(p).map_err(|e| result::Error::Class(e))?;
    type_check::run(p).map_err(|e| result::Error::Type(e))?;
    return_check::run(p).map_err(|e| result::Error::Return(e))?;
    Ok(())
}
