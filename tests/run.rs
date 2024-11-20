use anyhow::Result;
use assert_fs::prelude::*;
use insta::assert_snapshot;

use crate::common::{cmd_snapshot, TestContext};

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
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing main.py
    fix end of files.........................................................Failed
    - hook id: end-of-file-fixer
    - exit code: 1
    - files were modified by this hook
      Fixing invalid.json
      Fixing valid.json
      Fixing main.py
    check json...............................................................Passed

    ----- stderr -----
    "#);

    cmd_snapshot!(context.filters(), context.run().arg("trailing-whitespace"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    trim trailing whitespace.................................................Passed

    ----- stderr -----
    "#);

    cmd_snapshot!(context.filters(), context.run().arg("typos").arg("--hook-stage").arg("pre-push"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    No hook found for id `typos` and stage `pre-push`
    "#);

    Ok(())
}

#[test]
fn local() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    let cwd = context.workdir();
    cwd.child(".pre-commit-config.yaml")
        .write_str(indoc::indoc! {r"
            repos:
              - repo: local
                hooks:
                  - id: local
                    name: local
                    language: system
                    entry: echo Hello, world!
                    always_run: true
        "})?;

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    local....................................................................Passed

    ----- stderr -----
    "#);

    Ok(())
}

#[test]
fn local_need_install() -> Result<()> {
    let context = TestContext::new();
    context.init_project();

    let cwd = context.workdir();
    cwd.child("pyproject.toml").write_str(indoc::indoc! {r#"
        [project]
        name = "project"
        version = "0.1.0"
        requires-python = ">=3.12"
        dependencies = []

        [build-system]
        requires = ["hatchling"]
        build-backend = "hatchling.build"

        [project.scripts]
        hello = "project:main"
        "#})?;
    cwd.child("src/project").create_dir_all()?;
    cwd.child("src/project/__init__.py")
        .write_str(indoc::indoc! { r#"
        def main() -> None:
            print("Hello from project!")
    "#})?;

    cwd.child(".pre-commit-config.yaml")
        .write_str(indoc::indoc! {r"
            repos:
              - repo: local
                hooks:
                  - id: local
                    name: local
                    language: python
                    entry: hello
                    always_run: true
        "})?;

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: Local hook local does not need env
    "#);

    Ok(())
}

#[test]
fn invalid_hook_id() {
    let context = TestContext::new();

    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().arg("invalid-hook-id"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0

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
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
            "
    });

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
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                    name: 去除行尾空格
                  - id: end-of-file-fixer
                  - id: check-json
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    去除行尾空格.............................................................Passed
    fix end of files.........................................................Passed
    check json...........................................(no files to check)Skipped

    ----- stderr -----
    "#);
}

/// Skips hooks based on the `SKIP` environment variable.
#[test]
fn skips() {
    let context = TestContext::new();

    context.init_project();

    context.write_pre_commit_config(indoc::indoc! {r"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                  - id: end-of-file-fixer
                  - id: check-json
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().env("SKIP", "end-of-file-fixer"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Passed
    fix end of files........................................................Skipped
    check json...........................................(no files to check)Skipped

    ----- stderr -----
    "#);

    cmd_snapshot!(context.filters(), context.run().env("SKIP", "trailing-whitespace,end-of-file-fixer"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    trim trailing whitespace................................................Skipped
    fix end of files........................................................Skipped
    check json...........................................(no files to check)Skipped

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
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                  - id: end-of-file-fixer
                  - id: check-json
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing file.txt
    fix end of files.........................................................Passed
    check json...........................................(no files to check)Skipped

    ----- stderr -----
    "#);

    // Override hook level files and exclude.
    // Global files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                    files: valid.json
                  - id: end-of-file-fixer
                    exclude: (valid.json|main.py)
                  - id: check-json
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing valid.json
    fix end of files.........................................................Passed
    check json...............................................................Passed

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

    // Global files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r#"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                    types: [ "json" ]
                  - id: trailing-whitespace
                    types_or: [ "json", "python" ]
                  - id: trailing-whitespace
                    exclude_types: [ "json" ]
                  - id: trailing-whitespace
                    types: [ "json" ]
                    exclude_types: [ "json" ]
            "#
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing json.json
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing main.py
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing file.txt
    trim trailing whitespace.............................(no files to check)Skipped

    ----- stderr -----
    "#);

    Ok(())
}

/// Abort the run if a hook fails.
#[test]
fn fail_fast() -> Result<()> {
    let context = TestContext::new();

    context.init_project();

    let cwd = context.workdir();
    cwd.child("file.txt").write_str("Hello, world!  ")?;
    cwd.child("json.json").write_str("{}\n  ")?;
    cwd.child("main.py").write_str(r#"print "abc"  "#)?;

    // Global files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r#"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                    fail_fast: false
                    types: [ "json" ]
                  - id: trailing-whitespace
                    fail_fast: true
                  - id: trailing-whitespace
                  - id: trailing-whitespace
            "#
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing json.json
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing file.txt
      Fixing main.py

    ----- stderr -----
    "#);

    Ok(())
}

/// Run from a subdirectory. File arguments should be fixed to be relative to the root.
#[test]
fn subdirectory() -> Result<()> {
    let context = TestContext::new();

    context.init_project();

    let cwd = context.workdir();
    let child = cwd.child("foo/bar/baz");
    child.create_dir_all()?;
    child.child("file.txt").write_str("Hello, world!  ")?;

    // Global files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run().current_dir(&child).arg("--files").arg("file.txt"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook
      Fixing foo/bar/baz/file.txt

    ----- stderr -----
    "#);

    Ok(())
}

/// Test hook `log_file` option.
#[test]
fn log_file() -> Result<()> {
    let context = TestContext::new();

    context.init_project();

    let cwd = context.workdir();
    cwd.child("file.txt").write_str("Hello, world!  ")?;

    // Global files and exclude.
    context.write_pre_commit_config(indoc::indoc! {r"
            repos:
              - repo: https://github.com/pre-commit/pre-commit-hooks
                rev: v5.0.0
                hooks:
                  - id: trailing-whitespace
                    log_file: log.txt
            "
    });

    context.git_add(".");

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    Cloning https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    Installing environment for https://github.com/pre-commit/pre-commit-hooks@v5.0.0
    trim trailing whitespace.................................................Failed
    - hook id: trailing-whitespace
    - exit code: 1
    - files were modified by this hook

    ----- stderr -----
    "#);

    let log = context.read("log.txt");
    assert_snapshot!(log, @"Fixing file.txt");

    Ok(())
}

/// Pass pre-commit environment variables to the hook.
#[cfg(unix)]
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
                    entry: sh -c "echo $PRE_COMMIT > env.txt"
                    always_run: true
        "#});

    cmd_snapshot!(context.filters(), context.run(), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Pass environment.........................................................Passed

    ----- stderr -----
    "#);

    let env = context.read("env.txt");
    assert_eq!(env, "1\n");
}
