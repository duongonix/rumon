//! CLI argument parsing.

use std::path::PathBuf;

use rumon_config::CliOverrides;

use crate::commands::{CliCommand, RemoteCommand, WatchOutput};
use crate::error::CliError;

/// Parses Rumon CLI arguments.
///
/// # Errors
///
/// Returns an error when an option is unknown, an option value is missing, or a command separator
/// is present without a following command.
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliCommand, CliError> {
    let mut overrides = CliOverrides::default();
    let mut iterator = args.into_iter().peekable();

    while let Some(arg) = iterator.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(CliCommand::Help),
            "--version" | "-V" => return Ok(CliCommand::Version),
            "init" => return parse_init_command(&mut iterator),
            "tui" => return parse_tui_command(&mut iterator),
            "watch" => return parse_watch_command(&mut iterator),
            "server" => return parse_server_command(&mut iterator),
            "daemon" => return parse_daemon_command(&mut iterator),
            "remote" => return parse_remote_command(&mut iterator),
            "--config" | "-c" => {
                overrides.config_path = Some(PathBuf::from(next_value(&mut iterator, &arg)?));
            }
            "--watch" | "-w" => {
                overrides
                    .watch_paths
                    .push(PathBuf::from(next_value(&mut iterator, &arg)?));
            }
            "--ignore" | "-i" => {
                overrides
                    .ignore_paths
                    .push(PathBuf::from(next_value(&mut iterator, &arg)?));
            }
            "--ext" | "-e" => {
                overrides.extensions.push(next_value(&mut iterator, &arg)?);
            }
            "--cmd" => {
                overrides.command = Some(next_value(&mut iterator, &arg)?);
            }
            "--cwd" => {
                overrides.cwd = Some(PathBuf::from(next_value(&mut iterator, &arg)?));
            }
            "--debounce" => {
                let value = next_value(&mut iterator, &arg)?;
                overrides.debounce_ms = Some(
                    value
                        .parse()
                        .map_err(|_| CliError::new("--debounce must be a number"))?,
                );
            }
            "--no-tui" => overrides.no_tui = true,
            "--clear" => overrides.clear_logs = true,
            "--once" => overrides.once = true,
            "--no-restart" => overrides.no_restart = true,
            "--verbose" | "-v" => overrides.verbose = true,
            "--quiet" | "-q" => overrides.quiet = true,
            "--" => {
                let command: Vec<String> = iterator.collect();
                if command.is_empty() {
                    return Err(CliError::new("expected command after --"));
                }
                overrides.command = Some(command.join(" "));
                break;
            }
            value if value.starts_with('-') => {
                return Err(CliError::new(format!("unknown option: {value}")));
            }
            value => {
                return Err(CliError::new(format!(
                    "unexpected argument before --: {value}"
                )));
            }
        }
    }

    Ok(CliCommand::Run(overrides))
}

fn parse_tui_command(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<CliCommand, CliError> {
    let mut overrides = CliOverrides::default();
    while let Some(arg) = iterator.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                overrides.config_path = Some(PathBuf::from(next_value(iterator, &arg)?));
            }
            value => return Err(CliError::new(format!("unknown tui option: {value}"))),
        }
    }
    Ok(CliCommand::Tui(overrides))
}

fn parse_watch_command(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<CliCommand, CliError> {
    let mut overrides = CliOverrides::default();
    let mut format = None;
    while let Some(arg) = iterator.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                overrides.config_path = Some(PathBuf::from(next_value(iterator, &arg)?));
            }
            "--json" => format = Some(WatchOutput::Json),
            "--ndjson" => format = Some(WatchOutput::Ndjson),
            "--once" => overrides.once = true,
            value => return Err(CliError::new(format!("unknown watch option: {value}"))),
        }
    }
    Ok(CliCommand::Watch {
        overrides,
        format: format.unwrap_or(WatchOutput::Ndjson),
    })
}

fn parse_server_command(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<CliCommand, CliError> {
    let mut overrides = CliOverrides::default();
    let mut host = None;
    let mut port = None;
    while let Some(arg) = iterator.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                overrides.config_path = Some(PathBuf::from(next_value(iterator, &arg)?));
            }
            "--host" => host = Some(next_value(iterator, &arg)?),
            "--port" => {
                let value = next_value(iterator, &arg)?;
                port = Some(
                    value
                        .parse()
                        .map_err(|_| CliError::new("--port must be a number"))?,
                );
            }
            value => return Err(CliError::new(format!("unknown server option: {value}"))),
        }
    }
    Ok(CliCommand::Server {
        overrides,
        host,
        port,
    })
}

fn parse_daemon_command(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<CliCommand, CliError> {
    let mut overrides = CliOverrides::default();
    let mut ipc = false;
    while let Some(arg) = iterator.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                overrides.config_path = Some(PathBuf::from(next_value(iterator, &arg)?));
            }
            "--ipc" => ipc = true,
            value => return Err(CliError::new(format!("unknown daemon option: {value}"))),
        }
    }
    Ok(CliCommand::Daemon { overrides, ipc })
}

fn parse_init_command(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<CliCommand, CliError> {
    if let Some(value) = iterator.next() {
        return Err(CliError::new(format!("unexpected init argument: {value}")));
    }
    Ok(CliCommand::Init)
}

fn parse_remote_command(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<CliCommand, CliError> {
    let subcommand = iterator
        .next()
        .ok_or_else(|| CliError::new("expected remote subcommand: agent or connect"))?;
    let mut address = "127.0.0.1:4040".to_string();
    let mut node = "local".to_string();
    let mut token = String::new();

    while let Some(arg) = iterator.next() {
        match arg.as_str() {
            "--addr" | "--address" => address = next_value(iterator, &arg)?,
            "--node" => node = next_value(iterator, &arg)?,
            "--token" => token = next_value(iterator, &arg)?,
            value => return Err(CliError::new(format!("unknown remote option: {value}"))),
        }
    }

    if token.is_empty() {
        return Err(CliError::new("remote --token is required"));
    }

    let command = match subcommand.as_str() {
        "agent" => RemoteCommand::Agent {
            address,
            node,
            token,
        },
        "connect" => RemoteCommand::Connect {
            address,
            node,
            token,
        },
        _ => {
            return Err(CliError::new(
                "expected remote subcommand: agent or connect",
            ));
        }
    };
    Ok(CliCommand::Remote(command))
}

fn next_value(
    iterator: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    option: &str,
) -> Result<String, CliError> {
    iterator
        .next()
        .ok_or_else(|| CliError::new(format!("missing value for {option}")))
}

#[cfg(test)]
mod tests {
    use super::parse_args;
    use crate::{CliCommand, WatchOutput};
    use std::path::PathBuf;

    #[test]
    fn parses_command_after_separator() {
        let command = parse_args(["-w", "src", "--", "cargo", "run"].map(str::to_string))
            .expect("arguments should parse");

        let CliCommand::Run(overrides) = command else {
            panic!("expected run command");
        };

        assert_eq!(overrides.watch_paths, vec![PathBuf::from("src")]);
        assert_eq!(overrides.command, Some("cargo run".to_string()));
    }

    #[test]
    fn parses_help() {
        assert_eq!(
            parse_args(["--help"].map(str::to_string)).expect("help should parse"),
            CliCommand::Help
        );
    }

    #[test]
    fn parses_init() {
        assert_eq!(
            parse_args(["init"].map(str::to_string)).expect("init should parse"),
            CliCommand::Init
        );
    }

    #[test]
    fn rejects_init_arguments() {
        assert!(parse_args(["init", "--force"].map(str::to_string)).is_err());
    }

    #[test]
    fn parses_remote_connect() {
        let command = parse_args(
            [
                "remote",
                "connect",
                "--addr",
                "127.0.0.1:5050",
                "--token",
                "secret",
            ]
            .map(str::to_string),
        )
        .expect("remote command should parse");

        assert_eq!(
            command,
            CliCommand::Remote(crate::RemoteCommand::Connect {
                address: "127.0.0.1:5050".to_string(),
                node: "local".to_string(),
                token: "secret".to_string(),
            })
        );
    }

    #[test]
    fn parses_watch_ndjson() {
        let command =
            parse_args(["watch", "--ndjson"].map(str::to_string)).expect("watch should parse");
        let CliCommand::Watch { format, .. } = command else {
            panic!("expected watch command");
        };
        assert_eq!(format, WatchOutput::Ndjson);
    }

    #[test]
    fn parses_server_port() {
        let command = parse_args(["server", "--port", "3717"].map(str::to_string))
            .expect("server should parse");
        let CliCommand::Server { port, .. } = command else {
            panic!("expected server command");
        };
        assert_eq!(port, Some(3717));
    }
}
