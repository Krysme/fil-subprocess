use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::info;
pub fn set_log()
{
    env_logger::init();
}

pub fn set_panic_hook(name: &'static str)
{
    std::panic::set_hook(Box::new(move |_| {
        let bt = backtrace::Backtrace::new();
        info!("{} panic occured, backtrace: {:?}", name, bt);
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

#[macro_export]
macro_rules! shape_dispatch {
    ($sector_size: ident, $fun: ident, $uuid: ident) => {{
        match $sector_size as u64 {
            filecoin_proofs::SECTOR_SIZE_2_KIB => $fun::<filecoin_proofs::SectorShape2KiB>(&$uuid),
            filecoin_proofs::SECTOR_SIZE_4_KIB => $fun::<filecoin_proofs::SectorShape4KiB>(&$uuid),
            filecoin_proofs::SECTOR_SIZE_16_KIB => {
                $fun::<filecoin_proofs::SectorShape16KiB>(&$uuid)
            }
            filecoin_proofs::SECTOR_SIZE_32_KIB => {
                $fun::<filecoin_proofs::SectorShape32KiB>(&$uuid)
            }
            filecoin_proofs::SECTOR_SIZE_8_MIB => $fun::<filecoin_proofs::SectorShape8MiB>(&$uuid),
            filecoin_proofs::SECTOR_SIZE_16_MIB => {
                $fun::<filecoin_proofs::SectorShape16MiB>(&$uuid)
            }
            filecoin_proofs::SECTOR_SIZE_512_MIB => {
                $fun::<filecoin_proofs::SectorShape512MiB>(&$uuid)
            }
            filecoin_proofs::SECTOR_SIZE_1_GIB => $fun::<filecoin_proofs::SectorShape1GiB>(&$uuid),
            filecoin_proofs::SECTOR_SIZE_32_GIB => {
                $fun::<filecoin_proofs::SectorShape32GiB>(&$uuid)
            }
            filecoin_proofs::SECTOR_SIZE_64_GIB => {
                $fun::<filecoin_proofs::SectorShape64GiB>(&$uuid)
            }
            _ => ::anyhow::bail!("shape not recognized"),
        }
    }};
}
