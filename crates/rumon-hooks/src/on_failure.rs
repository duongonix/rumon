//! `on_failure` hook helper.

use crate::{HookContext, HookExecutor, HookOutput, HookPoint, HookSet};

/// Runs the configured `on_failure` hook.
///
/// # Errors
///
/// Returns an I/O error when the hook command cannot be spawned or waited on.
pub fn run_on_failure(
    hooks: &HookSet,
    executor: &HookExecutor,
    mut context: HookContext,
) -> std::io::Result<Option<HookOutput>> {
    context.event = HookPoint::OnFailure;
    hooks
        .get(HookPoint::OnFailure)
        .filter(|hook| hook.is_enabled())
        .map(|hook| executor.run(hook, &context))
        .transpose()
}
