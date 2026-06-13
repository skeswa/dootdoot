//! Command-line shell for dootdoot.

fn main() {
    let _format = active_format();
}

fn active_format() -> &'static str {
    dootdoot_core::FORMAT_V1
}
