use log::error;

fn main() {
    env_logger::init();

    if let Err(e) = deep_sea::get_arg().and_then(deep_sea::run) {
        error!("{e}");
        std::process::exit(-1);
    }
}
