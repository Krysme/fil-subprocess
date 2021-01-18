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


fn main()
{
    utils::set_log();
    info!("lotus-c2 started");
    utils::set_panic_hook("c2");

    let r = std::panic::catch_unwind(|| run());

    match r {
        Ok(Ok(_)) => std::process::exit(0),
        Ok(Err(e)) => {
            info!("c2 subprocess error:\n{:?}", e);
            std::process::exit(255)
        }
        Err(e) => {
            info!("c2 panic: {:?}", e);
            std::process::exit(254)
        }
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

    info!("ready to read parameter: {:?}", uuid_path);
    let infile =
        File::open(&uuid_path).with_context(|| format!("cannot open file {:?}", uuid_path))?;
    info!("parameter file opened: {:?}", uuid_path);

    let data = serde_json::from_reader::<_, C2Param<Tree>>(infile)
        .context("failed to deserialize p2 params")?;

    let C2Param {
        porep_config,
        phase1_output,
        prover_id,
        sector_id,
    } = data;
    info!("{:?}: parameter serialized: {:?}", sector_id, uuid_path);

    let out = custom::c2::whole(porep_config, phase1_output, prover_id, sector_id)?;

    std::fs::write(uuid_path, &out.proof)
        .with_context(|| format!("{:?}: cannot write result to file", sector_id))?;
    Ok(())
}
