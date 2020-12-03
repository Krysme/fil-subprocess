use std::{fs::File, path::Path};

use anyhow::{Context, Result};
use filecoin_proofs::{storage_proofs::sector::SectorId, *};

use log::info;
use serde::{Deserialize, Serialize};
use utils::ParentParam;

mod utils;

#[derive(Serialize, Deserialize)]
struct C2Param<Tree: 'static + MerkleTreeTrait>
{
    porep_config: PoRepConfig,
    #[serde(bound(
        serialize = "SealCommitPhase1Output<Tree>: Serialize",
        deserialize = "SealCommitPhase1Output<Tree>: Deserialize<'de>"
    ))]
    phase1_output: SealCommitPhase1Output<Tree>,
    prover_id: ProverId,
    sector_id: SectorId,
}

macro_rules! shape_dispatch {
    ($sector_size: ident, $fun: ident, $uuid: ident) => {{
        match $sector_size as u64 {
            SECTOR_SIZE_2_KIB => $fun::<SectorShape2KiB>(&$uuid),
            SECTOR_SIZE_4_KIB => $fun::<SectorShape4KiB>(&$uuid),
            SECTOR_SIZE_16_KIB => $fun::<SectorShape16KiB>(&$uuid),
            SECTOR_SIZE_32_KIB => $fun::<SectorShape32KiB>(&$uuid),
            SECTOR_SIZE_8_MIB => $fun::<SectorShape8MiB>(&$uuid),
            SECTOR_SIZE_16_MIB => $fun::<SectorShape16MiB>(&$uuid),
            SECTOR_SIZE_512_MIB => $fun::<SectorShape512MiB>(&$uuid),
            SECTOR_SIZE_1_GIB => $fun::<SectorShape1GiB>(&$uuid),
            SECTOR_SIZE_32_GIB => $fun::<SectorShape32GiB>(&$uuid),
            SECTOR_SIZE_64_GIB => $fun::<SectorShape64GiB>(&$uuid),
            _ => ::anyhow::bail!("shape not recognized"),
        }
    }};
}

fn main()
{
    utils::set_log();
    utils::set_panic_hook();

    if let Err(e) = run() {
        info!("subprocess error:\n{:?}", e);
    }
}

fn run() -> Result<()>
{
    let ParentParam { uuid, sector_size } = utils::param_from_parent()?;
    shape_dispatch!(sector_size, c2, uuid)
}

pub fn c2<Tree: 'static + MerkleTreeTrait>(uuid: &str) -> Result<()>
{
    let param_folder = utils::param_folder().context("cannot get param folder")?;
    let uuid_path = Path::new(&param_folder).join(&uuid);

    let infile =
        File::open(&uuid_path).with_context(|| format!("cannot open file {:?}", uuid_path))?;

    let data = serde_json::from_reader::<_, C2Param<Tree>>(infile)
        .context("failed to deserialize p2 params")?;

    let C2Param {
        porep_config,
        phase1_output,
        prover_id,
        sector_id,
    } = data;

    let out = custom::c2::whole(porep_config, phase1_output, prover_id, sector_id)?;

    std::fs::write(uuid_path, &out.proof)
        .with_context(|| format!("{:?}: cannot write result to file", sector_id))?;
    Ok(())
}
