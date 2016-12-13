use ast::Program;

mod return_check;
mod return_error;
mod type_check;
mod type_context;
mod type_error;

pub mod result;

pub fn analyse(p: &Program) -> result::Result {
    type_check::run(p).map_err(|e| result::Error::Type(e))?;
    return_check::run(p).map_err(|e| result::Error::Return(e))?;
    Ok(())
}
