#![allow(dead_code, unreachable_pub)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_fs::fixture::{ChildPath, FileWriteStr, PathChild};
use etcetera::BaseStrategy;

use constants::env_vars::EnvVars;

pub struct TestContext {
    temp_dir: ChildPath,
    home_dir: ChildPath,

    /// Standard filters for this test context.
    filters: Vec<(String, String)>,

    // To keep the directory alive.
    #[allow(dead_code)]
    _root: tempfile::TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        let bucket = Self::test_bucket_dir();
        fs_err::create_dir_all(&bucket).expect("Failed to create test bucket");

        let root = tempfile::TempDir::new_in(bucket).expect("Failed to create test root directory");

        let temp_dir = ChildPath::new(root.path()).child("temp");
        fs_err::create_dir_all(&temp_dir).expect("Failed to create test working directory");

        let home_dir = ChildPath::new(root.path()).child("home");
        fs_err::create_dir_all(&home_dir).expect("Failed to create test home directory");

        let mut filters = Vec::new();

        filters.extend(
            Self::path_patterns(&temp_dir)
                .into_iter()
                .map(|pattern| (pattern, "[TEMP_DIR]/".to_string())),
        );
        filters.extend(
            Self::path_patterns(&home_dir)
                .into_iter()
                .map(|pattern| (pattern, "[HOME]/".to_string())),
        );

        let current_exe = assert_cmd::cargo::cargo_bin("prefligit");
        filters.extend(
            Self::path_patterns(&current_exe)
                .into_iter()
                .map(|pattern| (pattern, "[CURRENT_EXE]".to_string())),
        );

        Self {
            temp_dir,
            home_dir,
            filters,
            _root: root,
        }
    }

    pub fn test_bucket_dir() -> PathBuf {
        EnvVars::var(EnvVars::PREFLIGIT_INTERNAL__TEST_DIR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                etcetera::base_strategy::choose_base_strategy()
                    .expect("Failed to find base strategy")
                    .data_dir()
                    .join("prefligit")
                    .join("tests")
            })
    }

    /// Generate an escaped regex pattern for the given path.
    fn path_pattern(path: impl AsRef<Path>) -> String {
        format!(
            // Trim the trailing separator for cross-platform directories filters
            r"{}\\?/?",
            regex::escape(&path.as_ref().display().to_string())
                // Make separators platform agnostic because on Windows we will display
                // paths with Unix-style separators sometimes
                .replace(r"\\", r"(\\|\/)")
        )
    }

    /// Generate various escaped regex patterns for the given path.
    pub fn path_patterns(path: impl AsRef<Path>) -> Vec<String> {
        let mut patterns = Vec::new();

        // We can only canonicalize paths that exist already
        if path.as_ref().exists() {
            patterns.push(Self::path_pattern(
                path.as_ref()
                    .canonicalize()
                    .expect("Failed to create canonical path"),
            ));
        }

        // Include a non-canonicalized version
        patterns.push(Self::path_pattern(path));

        patterns
    }

    /// Read a file in the temporary directory
    pub fn read(&self, file: impl AsRef<Path>) -> String {
        fs_err::read_to_string(self.temp_dir.join(&file))
            .unwrap_or_else(|_| panic!("Missing file: `{}`", file.as_ref().display()))
    }

    pub fn command(&self) -> Command {
        let bin = assert_cmd::cargo::cargo_bin("prefligit");
        let mut cmd = Command::new(bin);
        cmd.current_dir(self.workdir());
        cmd.env(EnvVars::PREFLIGIT_HOME, &*self.home_dir);
        cmd.env(EnvVars::PREFLIGIT_INTERNAL__SORT_FILENAMES, "1");
        cmd
    }

    pub fn run(&self) -> Command {
        let mut command = self.command();
        command.arg("run");
        command
    }

    pub fn clean(&self) -> Command {
        let mut command = self.command();
        command.arg("clean");
        command
    }

    pub fn validate_config(&self) -> Command {
        let mut command = self.command();
        command.arg("validate-config");
        command
    }

    pub fn validate_manifest(&self) -> Command {
        let mut command = self.command();
        command.arg("validate-manifest");
        command
    }

    pub fn install(&self) -> Command {
        let mut command = self.command();
        command.arg("install");
        command
    }

    pub fn uninstall(&self) -> Command {
        let mut command = self.command();
        command.arg("uninstall");
        command
    }

    pub fn sample_config(&self) -> Command {
        let mut command = self.command();
        command.arg("sample-config");
        command
    }

    /// Standard snapshot filters _plus_ those for this test context.
    pub fn filters(&self) -> Vec<(&str, &str)> {
        // Put test context snapshots before the default filters
        // This ensures we don't replace other patterns inside paths from the test context first
        self.filters
            .iter()
            .map(|(p, r)| (p.as_str(), r.as_str()))
            .chain(INSTA_FILTERS.iter().copied())
            .collect()
    }

    /// Get the working directory for the test context.
    pub fn workdir(&self) -> &ChildPath {
        &self.temp_dir
    }

    /// Initialize a sample project for prefligit.
    pub fn init_project(&self) {
        Command::new("git")
            .arg("init")
            .arg("--initial-branch=master")
            .current_dir(&self.temp_dir)
            .assert()
            .success();
    }

    /// Configure git user and email.
    pub fn configure_git_author(&self) {
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Prefligit Test")
            .current_dir(&self.temp_dir)
            .assert()
            .success();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@prefligit.dev")
            .current_dir(&self.temp_dir)
            .assert()
            .success();
    }

    /// Run `git add`.
    pub fn git_add(&self, path: impl AsRef<OsStr>) {
        Command::new("git")
            .arg("add")
            .arg(path)
            .current_dir(&self.temp_dir)
            .assert()
            .success();
    }

    /// Run `git commit`.
    pub fn git_commit(&self, message: &str) {
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&self.temp_dir)
            .assert()
            .success();
    }

    /// Write a `.pre-commit-config.yaml` file in the temporary directory.
    pub fn write_pre_commit_config(&self, content: &str) {
        self.temp_dir
            .child(".pre-commit-config.yaml")
            .write_str(content)
            .expect("Failed to write pre-commit config");
    }
}

#[doc(hidden)] // Macro and test context only, don't use directly.
pub const INSTA_FILTERS: &[(&str, &str)] = &[
    // File sizes
    (r"(\s|\()(\d+\.)?\d+([KM]i)?B", "$1[SIZE]"),
    // Rewrite Windows output to Unix output
    (r"\\([\w\d]|\.\.)", "/$1"),
    (r"prefligit.exe", "prefligit"),
    // The exact message is host language dependent
    (
        r"Caused by: .* \(os error 2\)",
        "Caused by: No such file or directory (os error 2)",
    ),
    // Time seconds
    (r"(\d+\.)?\d+(ms|s)", "[TIME]"),
];

#[allow(unused_macros)]
macro_rules! cmd_snapshot {
    ($spawnable:expr, @$snapshot:literal) => {{
        cmd_snapshot!($crate::common::INSTA_FILTERS.iter().copied().collect::<Vec<_>>(), $spawnable, @$snapshot)
    }};
    ($filters:expr, $spawnable:expr, @$snapshot:literal) => {{
        let mut settings = insta::Settings::clone_current();
        for (matcher, replacement) in $filters {
            settings.add_filter(matcher, replacement);
        }
        let _guard = settings.bind_to_scope();
        insta_cmd::assert_cmd_snapshot!($spawnable, @$snapshot);
    }};
}

#[allow(unused_imports)]
pub(crate) use cmd_snapshot;
