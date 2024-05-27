use std::fs;

use clap::Parser;

use crate::{build_project, CmdExector, JsWorker};

#[derive(Debug, Parser)]
pub struct RunOpts {}

impl CmdExector for RunOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let content = fs::read_to_string(filename)?;
        let worker = JsWorker::try_new(&content)?;
        // TODO: normally this should run axum and let it load the worker
        worker.run("await handlers.hello()")?;

        Ok(())
    }
}
