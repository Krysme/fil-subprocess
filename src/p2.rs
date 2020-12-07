use filecoin_proofs::process;
use log::info;
fn main()
{
    env_logger::init();
    info!("subprocess started");
    std::panic::set_hook(Box::new(|_| {
        let bt = backtrace::Backtrace::new();
        info!("p2 panic occurred, backtrace: {:?}", bt);
    }));

    let r = std::panic::catch_unwind(|| process::p2_sub_launcher());

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
