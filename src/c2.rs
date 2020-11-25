use filecoin_proofs::process;
use log::info;

fn main() {
    env_logger::init();
    std::panic::set_hook(Box::new(|_| {
        let bt = backtrace::Backtrace::new();
        info!("panic occured, backtrace: {:?}", bt);
    }));

    if let Err(e) = process::c2_sub_launcher() {
        info!("{:?}", e);
        std::process::exit(255);
    }
}
