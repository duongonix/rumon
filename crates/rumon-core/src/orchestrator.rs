//! Core runtime orchestration.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rumon_config::Config;
use rumon_hooks::{EventHookExecution, EventHookRunner};
use rumon_runner::{CommandRunner, RunnerConfig};
use rumon_shared::{AppEvent, LogEntry, LogKind, ProcessEvent, WatchEvent};
use rumon_tui::{KeyCommand, TerminalScreen, TuiApp, TuiConfig, spawn_input_reader};
use rumon_watch::{WatchOptions, spawn_watcher};

use crate::error::{CoreError, CoreResult};
use crate::output::print_plain_event;
use crate::state::Runtime;

/// Runs Rumon with a fully resolved configuration.
///
/// # Errors
///
/// Returns an error when the runner cannot start, a process operation fails, or the event bus
/// disconnects unexpectedly.
pub fn run(config: &Config) -> CoreResult<u8> {
    let runner_config = RunnerConfig {
        command: config.run.cmd.clone(),
        cwd: config.run.cwd.clone(),
        kill_signal: config.run.kill_signal.clone(),
    };

    if config.once {
        let code = rumon_runner::run_once(&runner_config)
            .map_err(|error| CoreError::new(error.to_string()))?;
        return Ok(exit_code_to_u8(code));
    }

    run_watch_mode(config, runner_config)
}

/// Returns a short startup message used by tests and diagnostics.
#[must_use]
pub fn startup_message() -> &'static str {
    "Rumon core runtime ready"
}

fn run_watch_mode(config: &Config, runner_config: RunnerConfig) -> CoreResult<u8> {
    let mut runtime = Runtime::new();
    runtime.seed_change_details(&config.watch);
    let event_hook_runner = EventHookRunner::new(&config.event_hooks, config.profile.clone());
    log_event_hook_diagnostics(&event_hook_runner, &mut runtime);
    let mut tui = create_tui(config, &runtime)?;
    let mut runner = CommandRunner::new(runner_config, runtime.sender());
    runner
        .start()
        .map_err(|error| CoreError::new(error.to_string()))?;

    let _watcher = spawn_watcher(
        WatchOptions {
            paths: config.watch.paths.clone(),
            ignore: config.watch.ignore.clone(),
            extensions: config.watch.extensions.clone(),
            recursive: config.watch.recursive,
            follow_symlink: config.watch.follow_symlink,
        },
        Duration::from_millis(config.watch.debounce_ms),
        runtime.sender(),
    );

    loop {
        if let Some(tui) = &mut tui {
            while let Ok(command) = tui.commands.try_recv() {
                if handle_tui_command(command, &mut tui.app, &mut runtime, &mut runner)
                    .map_err(|error| CoreError::new(error.to_string()))?
                {
                    runner
                        .stop()
                        .map_err(|error| CoreError::new(error.to_string()))?;
                    return Ok(0);
                }
            }
            tui.sync_and_draw(&runtime)?;
        }

        if let Some(status) = runner
            .poll_exit()
            .map_err(|error| CoreError::new(error.to_string()))?
        {
            if !config.run.restart {
                return Ok(exit_code_to_u8(status.code().unwrap_or(1)));
            }
        }

        match runtime.bus.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                let mut events = vec![event];
                while let Ok(event) = runtime.bus.try_recv() {
                    events.push(event);
                }

                let should_restart = events
                    .iter()
                    .any(|event| matches!(event, AppEvent::Watch(WatchEvent::Changed(_))))
                    && config.run.restart;

                for event in events {
                    if tui.is_none() {
                        print_plain_event(&event, config.quiet);
                    }
                    let applied = runtime.apply_and_return(event);
                    run_event_hooks(&event_hook_runner, &applied, &mut runtime, config.verbose)?;
                }

                if let Some(tui) = &mut tui {
                    tui.sync_and_draw(&runtime)?;
                }

                if should_restart {
                    if config.run.clear_logs_on_restart {
                        runtime.state.logs.clear();
                    }
                    thread::sleep(Duration::from_millis(config.run.restart_delay_ms));
                    runner
                        .restart()
                        .map_err(|error| CoreError::new(error.to_string()))?;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(CoreError::new("event bus disconnected"));
            }
        }
    }
}

fn run_event_hooks(
    runner: &EventHookRunner,
    event: &AppEvent,
    runtime: &mut Runtime,
    verbose: bool,
) -> CoreResult<()> {
    let AppEvent::Watch(WatchEvent::Changed(change)) = event else {
        return Ok(());
    };
    if verbose {
        for decision in runner.decisions(change) {
            runtime.apply(AppEvent::Process(ProcessEvent::Log(LogEntry::new(
                LogKind::System,
                format!(
                    "event hook '{}' {}: {}",
                    decision.hook_name,
                    if decision.matched {
                        "matched"
                    } else {
                        "skipped"
                    },
                    decision.reason
                ),
            ))));
        }
    }
    let executions = runner
        .run_matching(change)
        .map_err(|error| CoreError::new(format!("event hook failed: {error}")))?;
    for execution in executions {
        log_event_hook_execution(runtime, &execution);
    }
    Ok(())
}

fn log_event_hook_diagnostics(runner: &EventHookRunner, runtime: &mut Runtime) {
    for diagnostic in runner.diagnostics() {
        runtime.apply(AppEvent::Process(ProcessEvent::Log(LogEntry::new(
            LogKind::System,
            format!(
                "event hook '{}' skipped: {}",
                diagnostic.hook_name, diagnostic.message
            ),
        ))));
    }
}

fn log_event_hook_execution(runtime: &mut Runtime, execution: &EventHookExecution) {
    for line in execution.output.stdout.lines() {
        runtime.apply(AppEvent::Process(ProcessEvent::Log(LogEntry::new(
            LogKind::Stdout,
            format!("[hook:{}] {line}", execution.name),
        ))));
    }
    for line in execution.output.stderr.lines() {
        runtime.apply(AppEvent::Process(ProcessEvent::Log(LogEntry::new(
            LogKind::Stderr,
            format!("[hook:{}] {line}", execution.name),
        ))));
    }
    if !execution.output.succeeded() {
        runtime.apply(AppEvent::Process(ProcessEvent::Log(LogEntry::new(
            LogKind::System,
            format!(
                "event hook '{}' exited with {:?}",
                execution.name, execution.output.exit_code
            ),
        ))));
    }
}

fn exit_code_to_u8(code: i32) -> u8 {
    u8::try_from(code).unwrap_or(1)
}

struct ActiveTui {
    app: TuiApp,
    screen: TerminalScreen,
    commands: std::sync::mpsc::Receiver<KeyCommand>,
}

impl ActiveTui {
    fn sync_and_draw(&mut self, runtime: &Runtime) -> CoreResult<()> {
        self.app.sync_state(runtime.state().clone());
        self.screen
            .draw(&self.app)
            .map_err(|error| CoreError::new(error.to_string()))
    }
}

fn create_tui(config: &Config, runtime: &Runtime) -> CoreResult<Option<ActiveTui>> {
    if !config.tui.enabled {
        return Ok(None);
    }

    let app = TuiApp::new(
        runtime.state().clone(),
        TuiConfig {
            left_panel_width: config.tui.left_panel_width,
            show_timestamp: config.tui.show_timestamp,
            auto_scroll_logs: config.tui.auto_scroll_logs,
        },
    );
    let mut screen = TerminalScreen::new(config.run.cmd.clone());
    screen
        .enter()
        .map_err(|error| CoreError::new(error.to_string()))?;
    Ok(Some(ActiveTui {
        app,
        screen,
        commands: spawn_input_reader(),
    }))
}

fn handle_tui_command(
    command: KeyCommand,
    app: &mut TuiApp,
    runtime: &mut Runtime,
    runner: &mut CommandRunner,
) -> Result<bool, rumon_runner::RunnerError> {
    match command {
        KeyCommand::Quit => return Ok(true),
        KeyCommand::Restart => runner.restart()?,
        KeyCommand::ClearLogs => {
            runtime.state.logs.clear();
            app.sync_state(runtime.state().clone());
        }
        command => {
            let _ = app.apply_command(command);
        }
    }
    Ok(false)
}
