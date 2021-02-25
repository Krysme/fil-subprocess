mod utils;
use std::{collections::BTreeMap, fs::File, path::Path};

use anyhow::{Context, Result};
use filecoin_proofs::{storage_proofs::sector::SectorId, *};
use log::info;
use serde::{Deserialize, Serialize};
use utils::ParentParam;

#[derive(Serialize, Deserialize)]
struct WindowPostParam<Tree: 'static + MerkleTreeTrait>
{
    post_config: PoStConfig,
    randomness: ChallengeSeed,
    #[serde(bound(
        serialize = "SealPreCommitPhase1Output<Tree>: Serialize",
        deserialize = "SealPreCommitPhase1Output<Tree>: Deserialize<'de>"
    ))]
    replicas: BTreeMap<SectorId, PrivateReplicaInfo<Tree>>,
    prover_id: ProverId,
}

fn main()
{
    utils::set_log();
    info!("p2 subprocess started");
    utils::set_panic_hook("window-post");
    match std::panic::catch_unwind(run) {
        Ok(Ok(_)) => std::process::exit(0),
        Ok(Err(e)) => {
            info!("window post subprocess error: {:?}", e);
            std::process::exit(255)
        }
        Err(e) => {
            info!("window post panic: {:?}", e);
            std::process::exit(254)
        }
    }
}

fn run() -> Result<()>
{
    let ParentParam { uuid, sector_size } = utils::param_from_parent()?;
    shape_dispatch!(sector_size, post, uuid)
}

pub fn post<Tree: 'static + MerkleTreeTrait>(uuid: &str) -> Result<()>
{
    let param_folder = utils::param_folder().context("cannot get param folder")?;
    let uuid_path = Path::new(&param_folder).join(&uuid);

    info!("ready to read parameter: {:?}", uuid_path);
    let infile =
        File::open(&uuid_path).with_context(|| format!("cannot open file {:?}", uuid_path))?;
    info!("parameter file opened: {:?}", uuid_path);

    let data = serde_json::from_reader::<_, WindowPostParam<Tree>>(infile)
        .context("failed to deserialize p2 params")?;

    let gpu_index = std::env::args()
        .nth(3)
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    let WindowPostParam {
        post_config,
        randomness,
        replicas,
        prover_id,
    } = data;
    info!("parameter serialized: {:?}", uuid_path);

    let out = filecoin_proofs::generate_window_post_inner(
        &post_config,
        &randomness,
        &replicas,
        prover_id,
        gpu_index,
    )?;

    std::fs::write(uuid_path, &out).context("cannot write result to file")?;
    Ok(())
}
