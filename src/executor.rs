pub mod throttled;
pub mod volts;

pub trait Executor {
    type Output;

    fn execute(&self) -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}
