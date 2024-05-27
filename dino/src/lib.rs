mod cli;
mod engine;
mod utils;

use enum_dispatch::enum_dispatch;

pub use cli::*;
pub use engine::*;
pub(crate) use utils::*;

pub const BUILD_DIR: &str = ".build";

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExector {
    async fn execute(self) -> anyhow::Result<()>;
}
