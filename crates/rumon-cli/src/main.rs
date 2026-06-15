//! Rumon command line entrypoint.

use std::env;
use std::process::ExitCode;

use rumon_cli::{CliCommand, RemoteCommand, WatchOutput, parse_args, print_help};
use rumon_remote::{RemoteAgentConfig, RemoteClientConfig, RemoteToken};

fn main() -> ExitCode {
    match parse_args(env::args().skip(1)) {
        Ok(CliCommand::Help) => {
            print_help();
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Version) => {
            println!("rumon {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Init) => run_init(),
        Ok(CliCommand::Tui(overrides) | CliCommand::Run(overrides)) => run_tui(overrides),
        Ok(CliCommand::Watch { overrides, format }) => run_watch(overrides, format),
        Ok(CliCommand::Server {
            overrides,
            host,
            port,
        }) => run_server(overrides, host, port),
        Ok(CliCommand::Daemon { overrides, ipc: _ }) => run_daemon(overrides),
        Ok(CliCommand::Remote(command)) => run_remote(command),
        Err(error) => {
            eprintln!("rumon: {error}");
            eprintln!("try `rumon --help` for usage");
            ExitCode::from(2)
        }
    }
}

fn run_tui(overrides: rumon_config::CliOverrides) -> ExitCode {
    match rumon_config::load(overrides) {
        Ok(config) => match rumon_core::run(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("rumon: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("rumon: invalid configuration: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_watch(overrides: rumon_config::CliOverrides, format: WatchOutput) -> ExitCode {
    match rumon_config::load(overrides) {
        Ok(config) => {
            let format = match format {
                WatchOutput::Json => rumon_core::StreamFormat::Json,
                WatchOutput::Ndjson => rumon_core::StreamFormat::Ndjson,
            };
            match rumon_core::run_watch_stream(&config, format) {
                Ok(code) => ExitCode::from(code),
                Err(error) => {
                    eprintln!("rumon watch: {error}");
                    ExitCode::from(1)
                }
            }
        }
        Err(error) => {
            eprintln!("rumon watch: invalid configuration: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_server(
    overrides: rumon_config::CliOverrides,
    host: Option<String>,
    port: Option<u16>,
) -> ExitCode {
    match rumon_config::load(overrides) {
        Ok(config) => match rumon_core::run_api_server(&config, host, port) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("rumon server: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("rumon server: invalid configuration: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_daemon(overrides: rumon_config::CliOverrides) -> ExitCode {
    match rumon_config::load(overrides) {
        Ok(config) => match rumon_core::run_ipc_daemon(&config) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("rumon daemon: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("rumon daemon: invalid configuration: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_init() -> ExitCode {
    match env::current_dir()
        .map_err(|error| format!("failed to read current directory: {error}"))
        .and_then(|directory| {
            rumon_config::init_config(&directory).map_err(|error| error.to_string())
        }) {
        Ok(result) if result.created => {
            println!("rumon: created {}", result.path.display());
            ExitCode::SUCCESS
        }
        Ok(result) => {
            eprintln!("rumon: {} already exists", result.path.display());
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("rumon init: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_remote(command: RemoteCommand) -> ExitCode {
    match run_remote_inner(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rumon remote: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_remote_inner(command: RemoteCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        RemoteCommand::Agent {
            address,
            node,
            token,
        } => {
            println!("rumon remote: agent listening on {address} as {node}");
            rumon_remote::run_agent_once(&RemoteAgentConfig {
                address,
                node_name: node,
                token: RemoteToken::new(token)?,
            })?;
        }
        RemoteCommand::Connect {
            address,
            node,
            token,
        } => {
            let report = rumon_remote::connect_remote(&RemoteClientConfig {
                address,
                node_name: node,
                token: RemoteToken::new(token)?,
                max_frames: None,
            })?;
            println!(
                "remote node {} {:?} frames={}",
                report.node.name, report.node.state, report.frames_received
            );
            for event in report.node.events {
                println!("event {} {}", event.kind, event.message);
            }
            for log in report.node.logs {
                println!("log {} {}", log.source, log.message);
            }
        }
    }
    Ok(())
}
