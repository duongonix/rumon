//! `on_success` hook helper.

use crate::{HookContext, HookExecutor, HookOutput, HookPoint, HookSet};

/// Runs the configured `on_success` hook.
///
/// # Errors
///
/// Returns an I/O error when the hook command cannot be spawned or waited on.
pub fn run_on_success(
    hooks: &HookSet,
    executor: &HookExecutor,
    mut context: HookContext,
) -> std::io::Result<Option<HookOutput>> {
    context.event = HookPoint::OnSuccess;
    hooks
        .get(HookPoint::OnSuccess)
        .filter(|hook| hook.is_enabled())
        .map(|hook| executor.run(hook, &context))
        .transpose()
}
