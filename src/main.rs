fn main() {
    if let Err(e) = fintop::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
