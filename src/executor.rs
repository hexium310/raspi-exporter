pub mod throttled;

#[cfg_attr(test, mockall::automock)]
pub trait Executor {
    fn execute(&self) -> impl Future<Output = anyhow::Result<String>> + Send;
}
