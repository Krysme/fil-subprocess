use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::info;

pub fn set_log()
{
    env_logger::init();
}

pub fn set_panic_hook()
{
    std::panic::set_hook(Box::new(|_| {
        let bt = backtrace::Backtrace::new();
        info!("panic occured, backtrace: {:?}", bt);
    }));
}

pub struct ParentParam
{
    pub sector_size: usize,
    pub uuid: String,
}

pub fn param_from_parent() -> Result<ParentParam>
{
    let mut args = std::env::args().skip(1).take(2);
    let uuid = args.next().context("cannot get uuid")?;
    let sector_size = args
        .next()
        .context("cannot get sector-size parameter")?
        .parse()
        .context("cannot parse sector-size")?;

    Ok(ParentParam { sector_size, uuid })
}

pub fn param_folder() -> Option<PathBuf>
{
    Some(Path::new(&std::env::var("WORKER_PATH").ok()?).join("param"))
}
