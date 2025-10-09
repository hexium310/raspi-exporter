use std::ffi::OsStr;

use tokio::process::Command;

pub struct CommandExecutor<S, I> {
    command: S,
    args: I,
}

pub trait Executor<S, I> {
    fn execute(&self) -> impl Future<Output = anyhow::Result<String>> + Send;
}

impl<S, I> CommandExecutor<S, I> {
    pub fn new(command: S, args: I) -> Self {
        Self {
            command,
            args,
        }
    }
}

impl<S, I> Executor<S, I> for CommandExecutor<S, I>
where 
    S: AsRef<OsStr> + Clone + Copy + Send + Sync,
    I: IntoIterator<Item = S> + Clone + Copy + Send + Sync,
{
    async fn execute(&self) -> anyhow::Result<String> {
        let output = Command::new(self.command).args(self.args).output().await?;
        if !output.status.success() {
            anyhow::bail!("")
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}

pub trait Parser {
    type Item;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item>;
}
