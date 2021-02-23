mod utils;
use log::info;

fn main() {
    utils::set_log();
    info!("p2 subprocess started");
    utils::set_panic_hook("window-post");
    println!("Hello World");
}
