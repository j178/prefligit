use std::collections::{HashMap, HashSet};

use clap::Parser;
use futures::StreamExt;

use crate::git::{intent_to_add_files, lfs_files};
use crate::hook::Hook;
use crate::run::CONCURRENCY;

enum FileFilter {
    NoFilter,
    Files(HashSet<String>),
}

impl FileFilter {
    fn contains(&self, path: &str) -> bool {
        match self {
            FileFilter::NoFilter => true,
            FileFilter::Files(files) => files.contains(path),
        }
    }
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    enforce_all: bool,
    #[arg(default_value = "500")]
    max_kb: u64,
}

pub(crate) async fn check_added_large_files(
    hook: &Hook,
    filenames: &[&String],
    _env_vars: &HashMap<&'static str, String>,
) -> anyhow::Result<(i32, Vec<u8>)> {
    let entry = shlex::split(&hook.entry).ok_or(anyhow::anyhow!("Failed to parse entry"))?;
    let args = Args::try_parse_from(entry.iter().chain(&hook.args))?;

    let filter = if args.enforce_all {
        FileFilter::NoFilter
    } else {
        let add_files: HashSet<_> = intent_to_add_files().await?.into_iter().collect();
        FileFilter::Files(add_files)
    };

    let lfs_files = lfs_files::<HashSet<String>>(filenames).await?;
    let mut tasks = futures::stream::iter(
        filenames
            .iter()
            .filter(|f| filter.contains(f))
            .filter(|f| !lfs_files.contains(f.as_str())),
    )
    .map(async |filename| {
        let size = fs_err::tokio::metadata(filename).await?.len();
        let size = size / 1024;
        if size > args.max_kb {
            anyhow::Ok(Some(format!(
                "{filename} ({size} KB) exceeds {} KB\n",
                args.max_kb
            )))
        } else {
            anyhow::Ok(None)
        }
    })
    .buffered(*CONCURRENCY);

    let mut code = 0;
    let mut output = Vec::new();

    while let Some(result) = tasks.next().await {
        if let Some(e) = result? {
            code = 1;
            output.extend(e.into_bytes());
        }
    }

    Ok((code, output))
}
