use std::process::Command;

use anyhow::Result;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::prelude::*;
use insta::assert_snapshot;

use crate::common::{TestContext, cmd_snapshot};

mod common;

#[test]
fn run_basic() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    let cwd = context.workdir();
    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: https://github.com/pre-commit/pre-commit-hooks
            rev: v5.0.0
            hooks:
              - id: trailing-whitespace
              - id: end-of-file-fixer
              - id: check-json
    "});

    // Create a repository with some files.
    cwd.child("file.txt").write_str("Hello, world!\n")?;
    cwd.child("valid.json").write_str("{}")?;
    cwd.child("invalid.json").write_str("{}")?;
    cwd.child("main.py").write_str(r#"print "abc"  "#)?;

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing main.py
    fix end of files.........................................................Failed
    - hook id: end-of-file-fixer
    - exit code: 1
    - files were modified by this hook
      Fixing valid.json
      Fixing invalid.json
      Fixing main.py
    check json...............................................................Passed

    ----- stderr -----
    "#);

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().arg("trailing-whitespace"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    trim trailing whitespace.................................................Passed

    ----- stderr -----
    "#);

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().arg("typos").arg("--hook-stage").arg("pre-push"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    No hook found for id `typos` and stage `pre-push`
    "#);

    Ok(())
}

/// Use same repo multiple times, with same or different revisions.
#[test]
fn same_repo() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    let cwd = context.workdir();
    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: https://github.com/pre-commit/pre-commit-hooks
            rev: v5.0.0
            hooks:
              - id: trailing-whitespace
          - repo: https://github.com/pre-commit/pre-commit-hooks
            rev: v5.0.0
            hooks:
              - id: trailing-whitespace
          - repo: https://github.com/pre-commit/pre-commit-hooks
            rev: v4.6.0
            hooks:
              - id: trailing-whitespace
    "});

    cwd.child("file.txt").write_str("Hello, world!\n")?;
    cwd.child("valid.json").write_str("{}")?;
    cwd.child("invalid.json").write_str("{}")?;
    cwd.child("main.py").write_str(r#"print "abc"  "#)?;
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing main.py
    trim trailing whitespace.................................................Passed
    trim trailing whitespace.................................................Passed

    ----- stderr -----
    "#);

    Ok(())
}

#[test]
fn local() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: local
            hooks:
              - id: local
                name: local
                language: system
                entry: echo Hello, world!
                always_run: true
    "});

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    local....................................................................Passed

    ----- stderr -----
    "#);
}

#[test]
fn local_need_install() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: local
                name: local
                language: python
                entry: pyecho Hello, world!
                additional_dependencies: ["pyecho-cli"]
                always_run: true
    "#});

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    local....................................................................Passed

    ----- stderr -----
    "#);
}

#[test]
fn meta_hooks() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    let cwd = context.workdir();
    cwd.child("file.txt").write_str("Hello, world!\n")?;
    cwd.child("valid.json").write_str("{}")?;
    cwd.child("invalid.json").write_str("{}")?;
    cwd.child("main.py").write_str(r#"print "abc"  "#)?;

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: meta
            hooks:
              - id: check-hooks-apply
              - id: check-useless-excludes
              - id: identity
          - repo: local
            hooks:
              - id: match-no-files
                name: match no files
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                files: ^nonexistent$
              - id: useless-exclude
                name: useless exclude
                language: system
                entry: python3 -c 'import sys; sys.exit(0)'
                exclude: $nonexistent^
    "});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Check hooks apply........................................................Failed
    - hook id: check-hooks-apply
    - exit code: 1
      match-no-files does not apply to this repository
    Check useless excludes...................................................Failed
    - hook id: check-useless-excludes
    - exit code: 1
      The exclude pattern "$nonexistent^" for useless-exclude does not match any files
    identity.................................................................Passed
    - hook id: identity
    - duration: [TIME]
      file.txt
      .pre-commit-config.yaml
      valid.json
      invalid.json
      main.py
    match no files.......................................(no files to check)Skipped
    useless exclude..........................................................Passed

    ----- stderr -----
    "#);

    Ok(())
}

#[test]
fn invalid_hook_id() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -V
    "});

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().arg("invalid-hook-id"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    No hook found for id `invalid-hook-id`
    "#);
}

/// `.pre-commit-config.yaml` is not staged.
#[test]
fn config_not_staged() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    context.workdir().child(".pre-commit-config.yaml").touch()?;
    context.git_add(".");

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -V
    "});

    cmd_snapshot!(context.filters(), context.run().arg("invalid-hook-id"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Your pre-commit configuration is unstaged.
    `git add .pre-commit-config.yaml` to fix this.
    "#);

    Ok(())
}

/// Test the output format for a hook with a CJK name.
#[test]
fn cjk_hook_name() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: 去除行尾空格
                language: system
                entry: python3 -V
              - id: end-of-file-fixer
                name: fix end of files
                language: system
                entry: python3 -V
    "});

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    去除行尾空格.............................................................Passed
    fix end of files.........................................................Passed

    ----- stderr -----
    "#);
}

/// Skips hooks based on the `SKIP` environment variable.
#[test]
fn skips() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c "exit(1)"
              - id: end-of-file-fixer
                name: fix end of files
                language: system
                entry: python3 -c "exit(1)"
              - id: check-json
                name: check json
                language: system
                entry: python3 -c "exit(1)"
    "#});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().env("SKIP", "end-of-file-fixer"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    fix end of files........................................................Skipped
    check json...............................................................Failed
    - hook id: check-json
    - exit code: 1

    ----- stderr -----
    "#);

    cmd_snapshot!(context.filters(), context.run().env("SKIP", "trailing-whitespace,end-of-file-fixer"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing-whitespace.....................................................Skipped
    fix end of files........................................................Skipped
    check json...............................................................Failed
    - hook id: check-json
    - exit code: 1

    ----- stderr -----
    "#);
}

/// Test global `files`, `exclude`, and hook level `files`, `exclude`.
#[test]
fn files_and_exclude() -> Result<()> {
    let context = TestContext::new();

    context.init_project();

    let cwd = context.workdir();
    cwd.child("file.txt").write_str("Hello, world!  \n")?;
    cwd.child("valid.json").write_str("{}\n  ")?;
    cwd.child("invalid.json").write_str("{}")?;
    cwd.child("main.py").write_str(r#"print "abc"  "#)?;

    // Global files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r"
        files: file.txt
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing whitespace
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                types: [text]
              - id: end-of-file-fixer
                name: fix end of files
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                types: [text]
              - id: check-json
                name: check json
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                types: [json]
    "});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      ['file.txt']
    fix end of files.........................................................Failed
    - hook id: end-of-file-fixer
    - exit code: 1
      ['file.txt']
    check json...........................................(no files to check)Skipped

    ----- stderr -----
    "#);

    // Override hook level files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r"
        files: file.txt
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing whitespace
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                files: valid.json
              - id: end-of-file-fixer
                name: fix end of files
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                exclude: (valid.json|main.py)
              - id: check-json
                name: check json
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
    "});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing whitespace..................................(no files to check)Skipped
    fix end of files.........................................................Failed
    - hook id: end-of-file-fixer
    - exit code: 1
      ['file.txt']
    check json...............................................................Failed
    - hook id: check-json
    - exit code: 1
      ['file.txt']

    ----- stderr -----
    "#);

    Ok(())
}

/// Test selecting files by type, `types`, `types_or`, and `exclude_types`.
#[test]
fn file_types() -> Result<()> {
    let context = TestContext::new();

    context.init_project();

    let cwd = context.workdir();
    cwd.child("file.txt").write_str("Hello, world!  ")?;
    cwd.child("json.json").write_str("{}\n  ")?;
    cwd.child("main.py").write_str(r#"print "abc"  "#)?;

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                types: ["json"]
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                types_or: ["json", "python"]
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                exclude_types: ["json"]
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'import sys; print(sys.argv[1:]); exit(1)'
                types: ["json" ]
                exclude_types: ["json"]
    "#});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      ['json.json']
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      ['main.py', 'json.json']
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      ['file.txt', '.pre-commit-config.yaml', 'main.py']
    trailing-whitespace..................................(no files to check)Skipped

    ----- stderr -----
    "#);

    Ok(())
}

/// Abort the run if a hook fails.
#[test]
fn fail_fast() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'print("Fixing files"); exit(1)'
                always_run: true
                fail_fast: false
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'print("Fixing files"); exit(1)'
                always_run: true
                fail_fast: true
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -V
                always_run: true
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -V
                always_run: true
    "#});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      Fixing files
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      Fixing files

    ----- stderr -----
    "#);
}

/// Run from a subdirectory. File arguments should be fixed to be relative to the root.
#[test]
fn subdirectory() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    let cwd = context.workdir();
    let child = cwd.child("foo/bar/baz");
    child.create_dir_all()?;

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'print("Hello"); exit(1)'
                always_run: true
    "#});

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().current_dir(&child).arg("--files").arg("file.txt"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
      Hello

    ----- stderr -----
    "#);

    Ok(())
}

/// Test hook `log_file` option.
#[test]
fn log_file() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'print("Fixing files"); exit(1)'
                always_run: true
                log_file: log.txt
    "#});
    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trailing-whitespace......................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1

    ----- stderr -----
    "#);

    let log = context.read("log.txt");
    assert_eq!(log, "Fixing files");
}

/// Pass pre-commit environment variables to the hook.
#[test]
fn pass_env_vars() {
    let context = TestContext::new();

    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: env-vars
                name: Pass environment
                language: system
                entry: python3 -c "import os, sys; print(os.getenv('PRE_COMMIT')); sys.exit(1)"
                always_run: true
    "#});

    cmd_snapshot!(context.filters(), context.run(), @r###"
    success: false
    exit_code: 1
    ----- stdout -----
    Pass environment.........................................................Failed
    - hook id: env-vars
    - exit code: 1
      1

    ----- stderr -----
    "###);
}

#[test]
fn staged_files_only() -> Result<()> {
    let context = TestContext::new();
    context.init_project();
    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'print(open("file.txt", "rt").read())'
                verbose: true
                types: [text]
   "#});

    context
        .workdir()
        .child("file.txt")
        .write_str("Hello, world!")?;
    context.git_add(".");

    // Non-staged files should be stashed and restored.
    context
        .workdir()
        .child("file.txt")
        .write_str("Hello world again!")?;

    let filters: Vec<_> = context
        .filters()
        .into_iter()
        .chain([(r"/\d+-\d+.patch", "/[TIME]-[PID].patch")])
        .collect();

    cmd_snapshot!(filters, context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    trailing-whitespace......................................................Passed
    - hook id: trailing-whitespace
    - duration: [TIME]
      Hello, world!

    ----- stderr -----
    Non-staged changes detected, saving to `[HOME]/patches/[TIME]-[PID].patch`

    Restored working tree changes from `[HOME]/patches/[TIME]-[PID].patch`
    "#);

    let content = context.read("file.txt");
    assert_snapshot!(content, @"Hello world again!");

    Ok(())
}

#[cfg(unix)]
#[test]
fn restore_on_interrupt() -> Result<()> {
    let context = TestContext::new();
    context.init_project();
    // The hook will sleep for 3 seconds.
    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'import time; open("out.txt", "wt").write(open("file.txt", "rt").read()); time.sleep(10)'
                verbose: true
                types: [text]
   "#});

    context
        .workdir()
        .child("file.txt")
        .write_str("Hello, world!")?;
    context.git_add(".");

    // Non-staged files should be stashed and restored.
    context
        .workdir()
        .child("file.txt")
        .write_str("Hello world again!")?;

    let mut child = context.run().spawn()?;
    let child_id = child.id();

    // Send an interrupt signal to the process.
    let handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            libc::kill(child_id as i32, libc::SIGINT)
        };
    });

    handle.join().unwrap();
    child.wait()?;

    let content = context.read("out.txt");
    assert_snapshot!(content, @"Hello, world!");

    let content = context.read("file.txt");
    assert_snapshot!(content, @"Hello world again!");

    Ok(())
}

/// When in merge conflict, runs on files that have conflicts fixed.
#[test]
fn merge_conflicts() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    // Create a merge conflict.
    let cwd = context.workdir();
    cwd.child("file.txt").write_str("Hello, world!")?;
    context.git_add(".");
    context.configure_git_author();
    context.git_commit("Initial commit");

    Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg("feature")
        .current_dir(cwd)
        .assert()
        .success();
    cwd.child("file.txt").write_str("Hello, world again!")?;
    context.git_add(".");
    context.git_commit("Feature commit");

    Command::new("git")
        .arg("checkout")
        .arg("master")
        .current_dir(cwd)
        .assert()
        .success();
    cwd.child("file.txt")
        .write_str("Hello, world from master!")?;
    context.git_add(".");
    context.git_commit("Master commit");

    Command::new("git")
        .arg("merge")
        .arg("feature")
        .current_dir(cwd)
        .assert()
        .code(1);

    context.write_pre_commit_config(indoc::indoc! {r"
        repos:
          - repo: local
            hooks:
              - id: trailing-whitespace
                name: trailing-whitespace
                language: system
                entry: python3 -c 'import sys; print(sorted(sys.argv[1:]))'
                verbose: true
    "});

    // Abort on merge conflicts.
    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    You have unmerged paths. Resolve them before running prefligit.
    "#);

    // Fix the conflict and run again.
    context.git_add(".");
    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    trailing-whitespace......................................................Passed
    - hook id: trailing-whitespace
    - duration: [TIME]
      ['.pre-commit-config.yaml', 'file.txt']

    ----- stderr -----
    "#);

    Ok(())
}

/// Local python hook with no additional dependencies.
#[test]
fn local_python_hook() {
    let context = TestContext::new();
    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: local-python-hook
                name: local-python-hook
                language: python
                entry: python3 -c 'import sys; print("Hello, world!"); sys.exit(1)'
    "#});

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    local-python-hook........................................................Failed
    - hook id: local-python-hook
    - exit code: 1
      Hello, world!

    ----- stderr -----
    "#);
}

/// Supports reading `pre-commit-config.yml` as well.
#[test]
fn alternate_config_file() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    context
        .workdir()
        .child(".pre-commit-config.yml")
        .write_str(indoc::indoc! {r#"
        repos:
          - repo: local
            hooks:
              - id: local-python-hook
                name: local-python-hook
                language: python
                entry: python3 -c 'import sys; print("Hello, world!")'
    "#})?;

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().arg("-v"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    local-python-hook........................................................Passed
    - hook id: local-python-hook
    - duration: [TIME]
      Hello, world!

    ----- stderr -----
    "#);

    Ok(())
}
