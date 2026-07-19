mod cli;

fn main() {
    // cli::run() parses args and performs the requested operation.
    // It returns false only when no subcommand was given.
    if !cli::run() {
        eprintln!("No command given. Run with --help to see available commands.");
        std::process::exit(2);
    }
}
