use std::cmp::max;
use std::sync::LazyLock;

use futures::StreamExt;
use tracing::trace;

use constants::env_vars::EnvVars;

use crate::hook::Hook;

pub static CONCURRENCY: LazyLock<usize> = LazyLock::new(|| {
    if EnvVars::is_set(EnvVars::PREFLIGIT_NO_CONCURRENCY) {
        1
    } else {
        std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1)
    }
});

fn target_concurrency(serial: bool) -> usize {
    if serial { 1 } else { *CONCURRENCY }
}

/// Iterator that yields partitions of filenames that fit within the maximum command line length.
struct Partitions<'a> {
    hook: &'a Hook,
    filenames: &'a [&'a String],
    concurrency: usize,
    current_index: usize,
    command_length: usize,
    max_per_batch: usize,
    max_cli_length: usize,
}

// TODO: do a more accurate calculation
impl<'a> Partitions<'a> {
    fn new(hook: &'a Hook, filenames: &'a [&'a String], concurrency: usize) -> Self {
        let max_per_batch = max(4, filenames.len().div_ceil(concurrency));
        // TODO: subtract the env size
        let max_cli_length = if cfg!(unix) {
            1 << 12
        } else {
            (1 << 15) - 2048 // UNICODE_STRING max - headroom
        };
        let command_length =
            hook.entry.len() + hook.args.iter().map(String::len).sum::<usize>() + hook.args.len();

        Self {
            hook,
            filenames,
            concurrency,
            current_index: 0,
            command_length,
            max_per_batch,
            max_cli_length,
        }
    }
}

impl<'a> Iterator for Partitions<'a> {
    // TODO: produce slices instead of Vec
    type Item = Vec<&'a String>;

    fn next(&mut self) -> Option<Self::Item> {
        // Handle empty filenames case
        if self.filenames.is_empty() && self.current_index == 0 {
            self.current_index = 1;
            return Some(vec![]);
        }

        if self.current_index >= self.filenames.len() {
            return None;
        }

        let mut current = Vec::new();
        let mut current_length = self.command_length + 1;

        while self.current_index < self.filenames.len() {
            let filename = self.filenames[self.current_index];
            let length = filename.len() + 1;

            if current_length + length > self.max_cli_length || current.len() >= self.max_per_batch
            {
                break;
            }

            current.push(filename);
            current_length += length;
            self.current_index += 1;
        }

        if current.is_empty() {
            None
        } else {
            Some(current)
        }
    }
}

pub async fn run_by_batch<T, F>(
    hook: &Hook,
    filenames: &[&String],
    run: F,
) -> anyhow::Result<Vec<T>>
where
    F: AsyncFn(Vec<String>) -> anyhow::Result<T>,
    T: Send + 'static,
{
    let concurrency = target_concurrency(hook.require_serial);

    // Split files into batches
    let partitions = Partitions::new(hook, filenames, concurrency);
    trace!(
        total_files = filenames.len(),
        concurrency = concurrency,
        "Running {}",
        hook.id,
    );

    let mut tasks = futures::stream::iter(partitions)
        .map(|batch| {
            // TODO: avoid this allocation
            let batch: Vec<_> = batch.into_iter().map(ToString::to_string).collect();
            run(batch)
        })
        .buffered(concurrency);

    let mut results = Vec::new();
    while let Some(result) = tasks.next().await {
        results.push(result?);
    }

    Ok(results)
}
