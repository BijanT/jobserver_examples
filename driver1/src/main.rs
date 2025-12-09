use clap::{arg};
use spurs::{cmd, Execute};

fn run_setup(remote_shell: &spurs::SshShell) -> Result<(), libscail::ScailError>
{
    // Install dependencies via apt
    let dependencies = [
        "build-essential",
        "bison",
        "flex",
    ];
    remote_shell.run(cmd!("sudo apt update"))?;
    remote_shell.run(cmd!("sudo apt upgrade -y"))?;
    remote_shell.run(cmd!("sudo apt install -y {}", dependencies.join(" ")))?;

    Ok(())
}

fn run() -> Result<(), libscail::ScailError> {
    // Define the command line arguments for the driver program
    let matches = clap::Command::new("driver1")
        .about("First example driver program that does basic setup on an Ubuntu machine.")
        // The jobserver add the "--print_results_path" argument. This is a
        // holdover from a time long passed, and we should probably update the
        // jobserver to not do that anymore. But for now, we add this so our
        // driver does not complain about it.
        .arg(arg!(--print_results_path "Obselete"))
        .arg(arg!(<hostname> "The domain:port of the remote machine"))
        .arg(arg!(<username> "The username to use for SSH login"))
        // Currently, the driver is only doing setup, but we are breaking it
        // into its own subcommand because we will be adding others later on.
        .subcommand(
            clap::Command::new("setup")
                .about("Setup a fresh machine.")
        )
        .subcommand_required(true)
        .get_matches();

    // Get the SSH information from the command line arguments
    let hostname = matches.get_one::<String>("hostname").unwrap();
    let username = matches.get_one::<String>("username").unwrap();

    // Attempt to establish an SSH connection with the remote host, using any
    // public key in the ~/.ssh/ directory for validation.
    let remote_shell = spurs::SshShell::with_any_key(username, hostname)?;

    match matches.subcommand() {
        Some(("setup", _)) => run_setup(&remote_shell),
        _ => unreachable!(),
    }
}

fn main() {
    // Allow Rust to capture backtraces. This can be useful to debug errors.
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // If an error occurred when running the driver, make sure to print
    // the error and backtrace so we can know what happened.
    if let Err(e) = run() {
        println!(
            "Encountered the following error:\n{}\n{}",
            e,
            e.backtrace(),
        );
    }
}
