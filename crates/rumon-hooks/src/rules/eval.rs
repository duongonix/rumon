//! Rule expression evaluator.

use std::fs;
use std::path::Path;

use regex::Regex;
use rumon_shared::{ChangeDetail, FileChange, display_path};

use crate::events::EventType;
use crate::rules::ast::{BinaryOp, Expr, UnaryOp};
use crate::rules::error::RuleError;
use crate::rules::value::Value;

/// Runtime context used to evaluate a rule expression.
#[derive(Clone, Debug)]
pub struct RuleContext {
    change: FileChange,
    event_type: EventType,
}

impl RuleContext {
    /// Creates a rule context.
    #[must_use]
    pub const fn new(change: FileChange, event_type: EventType) -> Self {
        Self { change, event_type }
    }

    fn variable(&self, name: &str) -> Value {
        match name {
            "event.kind" => Value::String(self.event_type.as_str().to_string()),
            "event.path" | "event.new_path" => Value::String(normalize_path(&self.change.path)),
            "event.old_path" => Value::String(
                self.change
                    .previous_path
                    .as_ref()
                    .map_or_else(String::new, normalize_path),
            ),
            "event.is_file" => Value::Bool(!self.change.is_directory),
            "event.is_folder" => Value::Bool(self.change.is_directory),
            "file.name" => path_component(&self.change.path, Path::file_name),
            "file.stem" => path_component(&self.change.path, Path::file_stem),
            "file.ext" => path_component(&self.change.path, Path::extension),
            "file.size" => Value::Number(f64_from_u64(current_file_size(&self.change.path))),
            "file.size_delta" => Value::Number(f64_from_i64(size_delta(&self.change))),
            "file.exists" => Value::Bool(self.change.path.exists()),
            "file.is_binary" => Value::Bool(matches!(
                self.change.detail,
                Some(ChangeDetail::Binary { .. })
            )),
            "file.mime" => Value::String(mime_hint(&self.change.path)),
            "diff.added_lines" => Value::Number(f64_from_usize(diff_stats(&self.change).0)),
            "diff.removed_lines" => Value::Number(f64_from_usize(diff_stats(&self.change).1)),
            "diff.changed_lines" => {
                let (added, removed) = diff_stats(&self.change);
                Value::Number(f64_from_usize(added.saturating_add(removed)))
            }
            "diff.total_lines" => Value::Number(f64_from_usize(diff_total_lines(&self.change))),
            "diff.has_inline_changes" => Value::Bool(matches!(
                self.change.detail,
                Some(ChangeDetail::Text { .. })
            )),
            "metadata.permissions_changed" | "metadata.owner_changed" => Value::Bool(false),
            "metadata.modified_time_changed" => Value::Bool(matches!(
                self.change.kind,
                rumon_shared::ChangeKind::Modified
            )),
            "metadata.size_changed" => Value::Bool(size_delta(&self.change) != 0),
            "media.width" => {
                metadata_number(&self.change, "width").map_or(Value::Null, Value::Number)
            }
            "media.height" => {
                metadata_number(&self.change, "height").map_or(Value::Null, Value::Number)
            }
            "media.duration" => {
                metadata_number(&self.change, "duration").map_or(Value::Null, Value::Number)
            }
            "media.bitrate" => {
                metadata_number(&self.change, "bitrate").map_or(Value::Null, Value::Number)
            }
            "media.has_audio" => Value::Bool(
                media_kind(&self.change).is_some_and(|kind| kind == "audio" || kind == "video"),
            ),
            "media.has_video" => {
                Value::Bool(media_kind(&self.change).is_some_and(|kind| kind == "video"))
            }
            _ => Value::Null,
        }
    }
}

/// Evaluates an expression and returns its boolean truthiness.
///
/// # Errors
///
/// Returns an error for invalid operator/value combinations.
pub fn evaluate(expr: &Expr, context: &RuleContext) -> Result<bool, RuleError> {
    Ok(eval(expr, context)?.truthy())
}

fn eval(expr: &Expr, context: &RuleContext) -> Result<Value, RuleError> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Variable(name) => Ok(context.variable(name)),
        Expr::Array(items) => items
            .iter()
            .map(|item| eval(item, context))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Expr::Unary { op, expr } => {
            let value = eval(expr, context)?;
            match op {
                UnaryOp::Not => Ok(Value::Bool(!value.truthy())),
                UnaryOp::Neg => number_value(&value).map(|number| Value::Number(-number)),
            }
        }
        Expr::Binary { left, op, right } => eval_binary(left, *op, right, context),
    }
}

fn eval_binary(
    left: &Expr,
    op: BinaryOp,
    right: &Expr,
    context: &RuleContext,
) -> Result<Value, RuleError> {
    if op == BinaryOp::And {
        let left = eval(left, context)?;
        return if left.truthy() {
            Ok(Value::Bool(eval(right, context)?.truthy()))
        } else {
            Ok(Value::Bool(false))
        };
    }
    if op == BinaryOp::Or {
        let left = eval(left, context)?;
        return if left.truthy() {
            Ok(Value::Bool(true))
        } else {
            Ok(Value::Bool(eval(right, context)?.truthy()))
        };
    }

    let left = eval(left, context)?;
    let right = eval(right, context)?;
    match op {
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Ne => Ok(Value::Bool(left != right)),
        BinaryOp::Gt => compare_numbers(&left, &right, |a, b| a > b),
        BinaryOp::Ge => compare_numbers(&left, &right, |a, b| a >= b),
        BinaryOp::Lt => compare_numbers(&left, &right, |a, b| a < b),
        BinaryOp::Le => compare_numbers(&left, &right, |a, b| a <= b),
        BinaryOp::Add => arithmetic(&left, &right, |a, b| a + b),
        BinaryOp::Sub => arithmetic(&left, &right, |a, b| a - b),
        BinaryOp::Mul => arithmetic(&left, &right, |a, b| a * b),
        BinaryOp::Div => arithmetic(&left, &right, |a, b| a / b),
        BinaryOp::In => Ok(Value::Bool(
            matches!(right, Value::Array(items) if items.contains(&left)),
        )),
        BinaryOp::Contains => string_predicate(&left, &right, |left, right| left.contains(right)),
        BinaryOp::StartsWith => {
            string_predicate(&left, &right, |left, right| left.starts_with(right))
        }
        BinaryOp::EndsWith => string_predicate(&left, &right, |left, right| left.ends_with(right)),
        BinaryOp::Matches => regex_match(&left, &right),
        BinaryOp::And | BinaryOp::Or => unreachable!("handled before eager eval"),
    }
}

fn compare_numbers(
    left: &Value,
    right: &Value,
    compare: impl FnOnce(f64, f64) -> bool,
) -> Result<Value, RuleError> {
    Ok(Value::Bool(compare(
        number_value(left)?,
        number_value(right)?,
    )))
}

fn arithmetic(
    left: &Value,
    right: &Value,
    op: impl FnOnce(f64, f64) -> f64,
) -> Result<Value, RuleError> {
    Ok(Value::Number(op(number_value(left)?, number_value(right)?)))
}

fn number_value(value: &Value) -> Result<f64, RuleError> {
    value
        .as_number()
        .ok_or_else(|| RuleError::new("expected number"))
}

fn string_predicate(
    left: &Value,
    right: &Value,
    predicate: impl FnOnce(&str, &str) -> bool,
) -> Result<Value, RuleError> {
    Ok(Value::Bool(predicate(
        string_value(left)?,
        string_value(right)?,
    )))
}

fn regex_match(left: &Value, right: &Value) -> Result<Value, RuleError> {
    let regex =
        Regex::new(string_value(right)?).map_err(|error| RuleError::new(error.to_string()))?;
    Ok(Value::Bool(regex.is_match(string_value(left)?)))
}

fn string_value(value: &Value) -> Result<&str, RuleError> {
    value
        .as_str()
        .ok_or_else(|| RuleError::new("expected string"))
}

fn normalize_path(path: impl AsRef<Path>) -> String {
    display_path(path.as_ref())
}

fn path_component(path: &Path, component: impl FnOnce(&Path) -> Option<&std::ffi::OsStr>) -> Value {
    Value::String(
        component(path).map_or_else(String::new, |value| value.to_string_lossy().to_string()),
    )
}

fn current_file_size(path: &Path) -> u64 {
    fs::metadata(path).map_or(0, |metadata| metadata.len())
}

fn size_delta(change: &FileChange) -> i64 {
    match &change.detail {
        Some(ChangeDetail::Binary {
            previous_size: Some(previous),
            current_size: Some(current),
            ..
        }) => {
            i64::try_from(*current).unwrap_or(i64::MAX)
                - i64::try_from(*previous).unwrap_or(i64::MAX)
        }
        Some(ChangeDetail::Media { size_bytes, .. }) => {
            i64::try_from(*size_bytes).unwrap_or(i64::MAX)
        }
        _ => 0,
    }
}

fn mime_hint(path: &Path) -> String {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "rs" => "text/rust",
        "toml" => "text/toml",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        _ => "",
    }
    .to_string()
}

fn diff_stats(change: &FileChange) -> (usize, usize) {
    let Some(ChangeDetail::Text { preview, .. }) = &change.detail else {
        return (0, 0);
    };
    let added = preview.iter().filter(|line| line.starts_with('+')).count();
    let removed = preview.iter().filter(|line| line.starts_with('-')).count();
    (added, removed)
}

fn diff_total_lines(change: &FileChange) -> usize {
    match &change.detail {
        Some(ChangeDetail::Text { preview, .. }) => preview.len(),
        _ => 0,
    }
}

fn media_kind(change: &FileChange) -> Option<&str> {
    match &change.detail {
        Some(ChangeDetail::Media { kind, .. }) => Some(kind.as_str()),
        _ => None,
    }
}

fn metadata_number(change: &FileChange, key: &str) -> Option<f64> {
    let Some(ChangeDetail::Media { metadata, .. }) = &change.detail else {
        return None;
    };
    metadata.iter().find_map(|line| {
        let lower = line.to_ascii_lowercase();
        lower.contains(key).then(|| {
            lower
                .split(|character: char| !(character.is_ascii_digit() || character == '.'))
                .find(|part| !part.is_empty())
                .and_then(|part| part.parse().ok())
        })?
    })
}

fn f64_from_u64(value: u64) -> f64 {
    value.to_string().parse().unwrap_or(f64::MAX)
}

fn f64_from_i64(value: i64) -> f64 {
    value.to_string().parse().unwrap_or_else(|_| {
        if value.is_negative() {
            f64::MIN
        } else {
            f64::MAX
        }
    })
}

fn f64_from_usize(value: usize) -> f64 {
    value.to_string().parse().unwrap_or(f64::MAX)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rumon_shared::{ChangeDetail, ChangeKind, FileChange};

    use super::{RuleContext, evaluate};
    use crate::EventType;
    use crate::rules::parse_rule;

    #[test]
    fn evaluates_diff_and_extension_rule() {
        let expr = parse_rule(r#"diff.added_lines + diff.removed_lines >= 2 && file.ext == "rs""#)
            .expect("parse");
        let change = FileChange {
            path: PathBuf::from("src/main.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: None,
                preview: vec!["+ a".to_string(), "- b".to_string()],
                truncated: false,
            }),
        };

        assert!(evaluate(&expr, &RuleContext::new(change, EventType::FileModified)).expect("eval"));
    }
}
