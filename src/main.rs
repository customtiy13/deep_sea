fn main() {
    if let Err(e) = deep_sea::get_arg().and_then(deep_sea::run) {
        eprintln!("Error running {e}");
        std::process::exit(-1);
    }
}
