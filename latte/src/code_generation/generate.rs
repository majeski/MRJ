use code_generation::context::*;

pub trait GenerateCode<T> {
    fn generate_code(&self, ctx: &mut Context) -> T;
}
