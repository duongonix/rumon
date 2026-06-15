//! Lifecycle hook boundary for Rumon.

mod after_restart;
mod before_restart;
mod context;
mod events;
mod executor;
mod lifecycle;
mod matcher;
mod on_failure;
mod on_success;
mod result;
mod rules;
mod runner;

pub use after_restart::run_after_restart;
pub use before_restart::run_before_restart;
pub use context::HookContext;
pub use events::EventType;
pub use executor::{HookExecutor, run_hook};
pub use lifecycle::{Hook, HookPoint, HookSet};
pub use matcher::{EventHookDecision, EventHookDiagnostic, EventHookMatcher, MatchedEventHook};
pub use on_failure::run_on_failure;
pub use on_success::run_on_success;
pub use result::HookOutput;
pub use runner::{EventHookExecution, EventHookRunner};
