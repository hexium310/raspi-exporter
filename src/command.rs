use std::ffi::OsStr;

use tokio::process::Command;

pub trait Executor<S, I>
where 
    S: AsRef<OsStr> + Send,
    I: IntoIterator<Item = S> + Send,
{
    fn execute(&self, command: S, args: I) -> impl Future<Output = anyhow::Result<String>> + Send {
        async move {
            let output = Command::new(command).args(args).output().await?;
            if !output.status.success() {
                anyhow::bail!("")
            }

            let result = String::from_utf8(output.stdout)?;
            Ok(result)
        }
    }
}

pub trait Parser {
    type Item;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item>;
}
