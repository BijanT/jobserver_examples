use clap::{arg};
use spurs::{cmd, Execute};
use libscail::output::Parametrize;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Parametrize)]
struct ExpConfig {
    #[name]
    exp: String,

    time: u64,
    iterations: u64,

    #[timestamp]
    timestamp: libscail::output::Timestamp,
}

fn run_experiment(remote_shell: &spurs::SshShell, sub_m: &clap::ArgMatches) -> Result<(), libscail::ScailError>
{
    // Create the ExpConfig struct. It derives the libscail::output::Parametrize
    // trait, which is helpful for saving the parameters used in this experiment
    // for future reference and generating unique output filenames.
    let time = sub_m.get_one::<u64>("time").copied().unwrap();
    let iterations = sub_m.get_one::<u64>("iterations").copied().unwrap();
    let cfg = ExpConfig {
        exp: "demo_experiment".to_string(),
        time,
        iterations,
        timestamp: libscail::output::Timestamp::now(),
    };

    // Define the output files for this experiment
    let home_dir = libscail::get_user_home_dir(remote_shell)?;
    let results_dir = libscail::dir!(home_dir, "results/");
    let params_file = libscail::dir!(&results_dir, cfg.gen_file_name("params"));
    let time_file = libscail::dir!(&results_dir, cfg.gen_file_name("time"));

    // Make sure the results directory exists
    remote_shell.run(cmd!("mkdir -p {}", &results_dir))?;
    // Save the parameters of this experiment in case we need them later
    remote_shell.run(cmd!(
        "echo {} | tee {}",
        libscail::escape_for_bash(&serde_json::to_string(&cfg)?),
        params_file
    ))?;

    // Run some dummy experiment and save the result
    let start_time = std::time::Instant::now();
    for _ in 0..iterations {
        remote_shell.run(cmd!("sleep {}", time))?;
    }
    let elapsed_time = start_time.elapsed();
    remote_shell.run(cmd!(
        "echo {} | tee {}",
        elapsed_time.as_secs_f64(),
        time_file
    ))?;

    // The jobserver looks for the following in stdout of the driver program
    // to know what files to copy from the experiment machine.
    println!("RESULTS: {}", libscail::dir!(results_dir, cfg.gen_file_name("")));
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
        .subcommand(
            clap::Command::new("experiment")
                .about("Run the experiment")
                .arg(
                    arg!(--time <time> "The amount of time for each iteration")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    arg!(--iterations <iterations> "The number of iterations to run")
                        .value_parser(clap::value_parser!(u64)),
                )
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
        Some(("experiment", sub_m)) => run_experiment(&remote_shell, &sub_m),
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
