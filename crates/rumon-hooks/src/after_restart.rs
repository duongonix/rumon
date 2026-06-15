//! `after_restart` hook helper.

use crate::{HookContext, HookExecutor, HookOutput, HookPoint, HookSet};

/// Runs the configured `after_restart` hook.
///
/// # Errors
///
/// Returns an I/O error when the hook command cannot be spawned or waited on.
pub fn run_after_restart(
    hooks: &HookSet,
    executor: &HookExecutor,
    mut context: HookContext,
) -> std::io::Result<Option<HookOutput>> {
    context.event = HookPoint::AfterRestart;
    hooks
        .get(HookPoint::AfterRestart)
        .filter(|hook| hook.is_enabled())
        .map(|hook| executor.run(hook, &context))
        .transpose()
}
