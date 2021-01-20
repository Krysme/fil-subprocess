use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
use anyhow::Result;
use filecoin_proofs::{
    seal_pre_commit_phase2, MerkleTreeTrait, PoRepConfig, SealPreCommitPhase1Output,
};
use log::info;
mod utils;
use serde::Deserialize;
use serde::Serialize;
use utils::ParentParam;

#[derive(Serialize, Deserialize)]
struct P2Param<Tree: 'static + MerkleTreeTrait>
{
    porep_config: PoRepConfig,
    #[serde(bound(
        serialize = "SealPreCommitPhase1Output<Tree>: Serialize",
        deserialize = "SealPreCommitPhase1Output<Tree>: Deserialize<'de>"
    ))]
    phase1_output: SealPreCommitPhase1Output<Tree>,
    cache_path: PathBuf,
    replica_path: PathBuf,
}

fn main()
{
    utils::set_log();
    info!("p2 subprocess started");
    utils::set_panic_hook("p2");

    let r = std::panic::catch_unwind(|| run());
    match r {
        Ok(Ok(_)) => std::process::exit(0),
        Ok(Err(e)) => {
            info!("p2 subprocess error:\n{:?}", e);
            std::process::exit(255)
        }
        Err(e) => {
            info!("p2 panic: {:?}", e);
            std::process::exit(254)
        }
    }
}

fn run() -> Result<()>
{
    let ParentParam { uuid, sector_size } = utils::param_from_parent()?;
    shape_dispatch!(sector_size, p2, uuid)
}

pub fn p2<Tree: 'static + MerkleTreeTrait>(uuid: &str) -> Result<()>
{
    let param_folder = utils::param_folder().context("cannot get param folder")?;
    let uuid_path = Path::new(&param_folder).join(&uuid);

    info!("ready to read parameter: {:?}", uuid_path);
    let infile =
        File::open(&uuid_path).with_context(|| format!("cannot open file {:?}", uuid_path))?;
    info!("parameter file opened: {:?}", uuid_path);

    let data = serde_json::from_reader::<_, P2Param<Tree>>(infile)
        .context("failed to deserialize p2 params")?;

    let P2Param {
        porep_config,
        phase1_output,
        cache_path,
        replica_path,
    } = data;
    info!("{:?}: parameter serialized: {:?}", replica_path, uuid_path);

    let out = seal_pre_commit_phase2(porep_config, phase1_output, cache_path, &replica_path)?;

    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&out.comm_r);
    buf[32..].copy_from_slice(&out.comm_d);

    std::fs::write(uuid_path, &buf)
        .with_context(|| format!("{:?}: cannot write result to file", &replica_path))?;
    Ok(())
}
