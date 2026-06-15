#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <tag> [remote]"
  echo "Example: $0 v0.1.0 origin"
  exit 1
fi

TAG="$1"
REMOTE="${2:-origin}"
VERSION="${TAG#v}"

if ! command -v git >/dev/null 2>&1; then
  echo "error: git is required" >&2
  exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required" >&2
  exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([\-+][0-9A-Za-z.\-]+)?$ ]]; then
  echo "error: invalid tag/version '$TAG' (expected like v0.1.0)" >&2
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is not clean. Commit/stash first." >&2
  exit 1
fi

echo "==> Updating workspace version to $VERSION"
export REL_VERSION="$VERSION"
CURRENT_VERSION="$(perl -0777 -ne 'print $2 if /(^\s*\[workspace\.package\]\s*.*?^\s*version\s*=\s*")([^"]+)(")/ms' Cargo.toml)"
if [[ -z "$CURRENT_VERSION" ]]; then
  echo "error: failed to find [workspace.package] version in Cargo.toml" >&2
  exit 1
fi
perl -0777 -i -pe 's{(^\s*\[workspace\.package\]\s*.*?^\s*version\s*=\s*")([^"]+)(")}{$1 . $ENV{REL_VERSION} . $3}mse' Cargo.toml

if [[ "$CURRENT_VERSION" = "$VERSION" ]]; then
  echo "  workspace version is already $VERSION"
elif ! grep -q "version = \"$VERSION\"" Cargo.toml; then
  echo "error: failed to update workspace package version" >&2
  exit 1
fi

echo "==> Running checks"
cargo fmt --all --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

echo "==> Committing release changes"
git add Cargo.toml Cargo.lock
if [[ -z "$(git diff --cached --name-only)" ]]; then
  echo "  no version changes to commit"
else
  git commit -m "release: $TAG"
fi

echo "==> Tagging and pushing"
git tag "$TAG"
git push "$REMOTE" HEAD
git push "$REMOTE" "$TAG"

echo ""
echo "Release tag pushed: $TAG"
echo "GitHub release workflow should start automatically."
