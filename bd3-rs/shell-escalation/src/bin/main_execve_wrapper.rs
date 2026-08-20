#[cfg(not(unix))]
fn main() {
    eprintln!("bd3-execve-wrapper is only implemented for UNIX");
    std::process::exit(1);
}

#[cfg(unix)]
pub use bd3_shell_escalation::main_execve_wrapper as main;
