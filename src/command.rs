use std::{ffi::OsStr, fmt::Debug};

use anyhow::Context as _;
use futures::future;
use tokio::{process::Command};
use tracing::Level;

use crate::executor::Executor;

#[derive(Debug)]
pub struct CommandExecutor<S, I> {
    command: S,
    args: I,
}

#[derive(Debug)]
pub struct MultiCommandExecutor<S, I> {
    commands: Vec<CommandExecutor<S, I>>,
}

impl<S, I> CommandExecutor<S, I> {
    pub fn new(command: S, args: I) -> Self {
        Self {
            command,
            args,
        }
    }
}

impl<S, I> MultiCommandExecutor<S, I>
where
    S: Clone + Copy,
    I: Clone + Copy,
{
    pub fn new(commands: &[(S, I)]) -> Self {
        let commands = commands
            .iter()
            .map(|&(command, args)| CommandExecutor::new(command, args))
            .collect();
        Self { commands }
    }
}

impl<S, I> Executor for CommandExecutor<S, I>
where
    S: AsRef<OsStr> + Debug + Clone + Copy + Send + Sync,
    I: IntoIterator<Item = S> + Debug + Clone + Copy + Send + Sync,
{
    type Output = String;

    #[tracing::instrument(skip_all, fields(command = ?self.command, args = ?self.args), ret(level = Level::DEBUG))]
    async fn execute(&self) -> anyhow::Result<Self::Output> {
        execute(self).await
    }
}

impl<S, I> Executor for MultiCommandExecutor<S, I>
where
    S: AsRef<OsStr> + Debug + Clone + Copy + Send + Sync,
    I: IntoIterator<Item = S> + Debug + Clone + Copy + Send + Sync,
{
    type Output = Vec<String>;

    #[tracing::instrument(skip_all, fields(commands = ?self.commands), ret(level = Level::DEBUG))]
    async fn execute(&self) -> anyhow::Result<Self::Output> {
        let outputs = self.commands.iter().map(execute);
        future::try_join_all(outputs).await
    }
}

async fn execute<S, I>(executor: &CommandExecutor<S, I>) -> anyhow::Result<String>
where
    S: AsRef<OsStr> + Debug + Clone + Copy + Send + Sync,
    I: IntoIterator<Item = S> + Debug + Clone + Copy + Send + Sync,
{
    let output = Command::new(executor.command)
        .args(executor.args)
        .output()
        .await
        .with_context(|| format!("command execution error: {executor:?}"))?;
    if !output.status.success() {
        match output.status.code() {
            Some(code) => anyhow::bail!(format!("process exited with status code {code}: {executor:?}")),
            None => anyhow::bail!(format!("process terminated by signal: {executor:?}")),
        }
    }

    let result = String::from_utf8(output.stdout)?;
    Ok(result)
}
