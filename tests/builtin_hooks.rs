// FILE: ./tests/builtin_hooks.rs
use anyhow::Result;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::prelude::*;
use insta::assert_snapshot;

use crate::common::TestContext;

mod common;

/// A helper function to normalize the command output for snapshot testing.
/// It sorts the lines that start with "Fixing " to make the output deterministic.
fn normalize_output(output: &std::process::Output) -> String {
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = stdout_str.lines().collect();

    // Separate and sort the "Fixing..." lines from the rest of the output
    let (mut fixing_lines, other_lines): (Vec<_>, Vec<_>) = lines
        .drain(..)
        .partition(|&line| line.trim().starts_with("Fixing "));
    fixing_lines.sort_unstable();

    // Reconstruct the output with the sorted lines at the end
    let mut normalized_lines = other_lines;
    normalized_lines.extend(fixing_lines);

    normalized_lines.join("\n")
}

#[test]
fn end_of_file_fixer_hook() -> Result<()> {
    let context = TestContext::new();
    context.init_project();
    context.configure_git_author();

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: https://github.com/pre-commit/pre-commit-hooks
            rev: v5.0.0
            hooks:
              - id: end-of-file-fixer
    "});

    let cwd = context.workdir();

    // --- Create test files ---
    cwd.child("correct_lf.txt").write_str("Hello World\n")?;
    cwd.child("correct_crlf.txt").write_str("Hello World\r\n")?;
    cwd.child("no_newline.txt")
        .write_str("No trailing newline")?;
    cwd.child("multiple_lf.txt")
        .write_str("Multiple newlines\n\n\n")?;
    cwd.child("multiple_crlf.txt")
        .write_str("Multiple newlines\r\n\r\n")?;
    cwd.child("empty.txt").touch()?;
    cwd.child("only_newlines.txt").write_str("\n\n")?;
    cwd.child("only_win_newlines.txt").write_str("\r\n\r\n")?;

    context.git_add(".");

    // First run: hooks should fail and fix the files
    let first_run_output = context.run().assert().failure().get_output().clone();
    let normalized_first_run = normalize_output(&first_run_output);

    // Snapshot the normalized, deterministic output
    assert_snapshot!(normalized_first_run, @r###"
    fix end of files.........................................................Failed
    - hook id: end-of-file-fixer
    - exit code: 1
    - files were modified by this hook
      Fixing correct_crlf.txt
      Fixing multiple_crlf.txt
      Fixing multiple_lf.txt
      Fixing no_newline.txt
      Fixing only_newlines.txt
      Fixing only_win_newlines.txt
    "###);

    // Assert that the files have been corrected
    assert_snapshot!(context.read("correct_lf.txt"), @"Hello World\n");
    assert_snapshot!(context.read("correct_crlf.txt"), @"Hello World\n");
    assert_snapshot!(context.read("no_newline.txt"), @"No trailing newline\n");
    assert_snapshot!(context.read("multiple_lf.txt"), @"Multiple newlines\n");
    assert_snapshot!(context.read("multiple_crlf.txt"), @"Multiple newlines\n");
    assert_snapshot!(context.read("empty.txt"), @"");
    assert_snapshot!(context.read("only_newlines.txt"), @"\n");
    assert_snapshot!(context.read("only_win_newlines.txt"), @"\n");

    context.git_add(".");

    // Second run: hooks should now pass. The output will be stable.
    let second_run_output = context.run().assert().success().get_output().clone();
    let normalized_second_run = normalize_output(&second_run_output);

    assert_snapshot!(normalized_second_run, @"fix end of files.........................................................Passed");

    Ok(())
}
