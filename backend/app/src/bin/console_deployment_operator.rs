//! Separately authenticated deployment operation; never starts serving state.
fn main() -> std::process::ExitCode {
    std::panic::set_hook(Box::new(|_| {
        eprintln!("deployment_operator.process_failed; reconcile the same command");
    }));
    std::process::ExitCode::from(console_app::deployment_operator::run(
        std::env::args_os().skip(1),
    ))
}
