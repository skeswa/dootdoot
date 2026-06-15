//! Build-time tooling for dootdoot.

fn main() {
    match xtask::run() {
        Ok(output) if output.is_empty() => {}
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
