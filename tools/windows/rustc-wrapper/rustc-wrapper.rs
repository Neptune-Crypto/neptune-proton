use std::env;
use std::process::Command;

fn main() {
    // Collect args. 
    // args[0] is this wrapper's path.
    // args[1] is the path to the real rustc (provided by Cargo).
    // args[2..] are the arguments intended for rustc.
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: RUSTC_WRAPPER must be called with the path to rustc as the first argument.");
        std::process::exit(1);
    }

    let real_rustc = &args[1];
    let rustc_args = &args[2..];

    // execute rustc with the original args + the stack size flag
    let status = Command::new(real_rustc)
        .args(rustc_args)
        .arg("-C")
        .arg("link-arg=/STACK:33554432") // 32MB Stack
        .status()
        .expect("Failed to execute rustc");

    // Exit with the same code as rustc
    std::process::exit(status.code().unwrap_or(1));
}
