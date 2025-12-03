use anyhow::Result;

use crate::generator::context::GeneratorContext;

#[allow(async_fn_in_trait)]
pub trait Generator<T> {
    async fn execute(&self, context: GeneratorContext) -> Result<T>;
}
