use std::collections::HashMap;
use std::str::FromStr;

use crate::builtin::pre_commit_hooks::{Implemented, is_pre_commit_hooks};
use crate::hook::{Hook, Repo};

mod meta_hooks;
mod pre_commit_hooks;

/// Returns true if the hook has a builtin Rust implementation.
pub fn check_fast_path(hook: &Hook) -> bool {
    match hook.repo() {
        Repo::Meta { .. } => true,
        Repo::Remote { url, .. } if is_pre_commit_hooks(url) => {
            Implemented::from_str(hook.id.as_str()).is_ok()
        }
        _ => false,
    }
}

pub async fn run_fast_path(
    hook: &Hook,
    filenames: &[&String],
    env_vars: &HashMap<&'static str, String>,
) -> anyhow::Result<(i32, Vec<u8>)> {
    match hook.repo() {
        Repo::Meta { .. } => run_meta_hook(hook, filenames, env_vars).await,
        Repo::Remote { url, .. } if is_pre_commit_hooks(url) => {
            Implemented::from_str(hook.id.as_str())
                .unwrap()
                .run(hook, filenames, env_vars)
                .await
        }
        _ => unreachable!(),
    }
}

async fn run_meta_hook(
    hook: &Hook,
    filenames: &[&String],
    env_vars: &HashMap<&'static str, String>,
) -> anyhow::Result<(i32, Vec<u8>)> {
    match hook.id.as_str() {
        "check-hooks-apply" => meta_hooks::check_hooks_apply(hook, filenames, env_vars).await,
        "check-useless-excludes" => {
            meta_hooks::check_useless_excludes(hook, filenames, env_vars).await
        }
        "identity" => Ok(meta_hooks::identity(hook, filenames, env_vars)),
        _ => unreachable!(),
    }
}
