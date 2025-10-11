use std::{ffi::OsStr, fmt::Debug};

use anyhow::Context;
use tokio::process::Command;

#[derive(Debug)]
pub struct CommandExecutor<S, I> {
    command: S,
    args: I,
}

#[cfg_attr(test, mockall::automock)]
pub trait Executor {
    fn execute(&self) -> impl Future<Output = anyhow::Result<String>> + Send;
}

pub trait Parser {
    type Item;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item>;
}

impl<S, I> CommandExecutor<S, I> {
    pub fn new(command: S, args: I) -> Self {
        Self {
            command,
            args,
        }
    }
}

impl<S, I> Executor for CommandExecutor<S, I>
where
    S: AsRef<OsStr> + Debug + Clone + Copy + Send + Sync,
    I: IntoIterator<Item = S> + Debug + Clone + Copy + Send + Sync,
{
    async fn execute(&self) -> anyhow::Result<String> {
        let output = Command::new(self.command)
            .args(self.args)
            .output()
            .await
            .context(format!("{self:?}"))?;
        if !output.status.success() {
            anyhow::bail!("")
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}
