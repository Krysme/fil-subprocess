use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use filecoin_proofs::{storage_proofs::sector::SectorId, *};

use log::info;
use serde::{Deserialize, Serialize};
use utils::ParentParam;

mod utils;

#[derive(Serialize, Deserialize)]
struct P1Param {
    porep_config: PoRepConfig,
    cache_path: PathBuf,
    in_path: PathBuf,
    prover_id: ProverId,
    sector_id: SectorId,
    ticket: Ticket,
    piece_infos: Vec<PieceInfo>,
}

fn main() {
    if let Err(e) = utils::unbind_cores() {
        info!("cannot unbind cores: {:?}", e);
        std::process::exit(255);
    }
    info!("cores unbound for c2");

    utils::set_log();
    info!("lotus-c2 started");
    utils::set_panic_hook("p1");

    let r = std::panic::catch_unwind(run);

    match r {
        Ok(Ok(_)) => std::process::exit(0),
        Ok(Err(e)) => {
            info!("c2 subprocess error: {:?}", e);
            info!("c2 subprocess backtrace: {:?}", e.backtrace());

            utils::param_from_parent()
                .and_then(|x| Ok((utils::param_folder().context("cannot get param folder")?, x)))
                .map(|(folder, param)| Path::new(&folder).join(&param.uuid))
                .and_then(|path| {
                    std::fs::write(path, format!("error {:?}\nbacktrace: {}", e, e.backtrace()))
                        .context("cannot serialize error to uuid file")
                })
                .unwrap_or_else(|e| info!("cannot report error: {:?}", e));

            std::process::exit(255)
        }
        Err(e) => {
            info!("c2 panic: {:?}", e);
            std::process::exit(254)
        }
    }
}

fn run() -> Result<()> {
    let ParentParam { uuid, sector_size } = utils::param_from_parent()?;
    shape_dispatch!(sector_size, p1, uuid)
}

pub fn p1<Tree: 'static + MerkleTreeTrait>(uuid: &str) -> Result<()> {
    let param_folder = utils::param_folder().context("cannot get param folder")?;
    let uuid_path = Path::new(&param_folder).join(&uuid);

    info!("ready to read parameter: {:?}", uuid_path);
    let infile =
        File::open(&uuid_path).with_context(|| format!("cannot open file {:?}", uuid_path))?;
    info!("parameter file opened: {:?}", uuid_path);

    let data =
        serde_json::from_reader::<_, P1Param>(infile).context("failed to deserialize p2 params")?;

    let P1Param {
        porep_config,
        cache_path,
        in_path,
        prover_id,
        sector_id,
        ticket,
        piece_infos,
    } = data;
    info!("{:?}: parameter serialized: {:?}", sector_id, uuid_path);

    let out = seal_pre_commit_phase1_layer::<_, _, _, Tree>(
        porep_config,
        cache_path,
        in_path,
        PathBuf::default(),
        prover_id,
        sector_id,
        ticket,
        &piece_infos,
    )?;

    std::fs::write(uuid_path, serde_json::to_string(&out).unwrap())
        .with_context(|| format!("{:?}: cannot write result to file", sector_id))?;
    Ok(())
}
