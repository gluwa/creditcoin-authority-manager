use crate::RpcTExt;
use crate::Run;
use crate::RunResult;
use crate::RuntimeApi;
use async_trait::async_trait;
use clap::{Args, Subcommand};

#[derive(Debug, Clone, Subcommand)]
pub enum LogFilterCommand {
    /// Add a new log filter directive.
    Add(AddLogFilterArgs),
    /// Reset the log filter to default.
    Reset(ResetLogFilterArgs),
}

#[derive(Debug, Clone, Args)]
pub struct AddLogFilterArgs {
    /// The log filter directive to add.
    filter: String,
}

#[async_trait]
impl Run for AddLogFilterArgs {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        let Self { filter } = self;

        api.client.rpc().add_log_filter(filter.clone()).await?;
        println!("Added log filter directive {}", filter);
        Ok(())
    }
}

#[async_trait]
impl Run for ResetLogFilterArgs {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        api.client.rpc().reset_log_filter().await?;
        println!("Reset log filter to default");
        Ok(())
    }
}

#[derive(Debug, Clone, Args)]
pub struct ResetLogFilterArgs;

#[async_trait]
impl Run for LogFilterCommand {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        match self {
            LogFilterCommand::Add(args) => args.run(api).await,
            LogFilterCommand::Reset(args) => args.run(api).await,
        }
    }
}
