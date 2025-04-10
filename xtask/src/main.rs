// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//!
//! Build driver for boot loader/debugger.
//!
use clap::Parser;
use duct::cmd;
use std::env;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "bldb",
    author = "Oxide Computer Company",
    version = "0.1.0",
    about = "xtask build tool for boot loader debugger"
)]
struct Xtask {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// Builds bldb
    Build {
        #[clap(flatten)]
        profile: BuildProfile,
        #[clap(flatten)]
        locked: Locked,
        #[clap(flatten)]
        features: Features,
    },
    /// cargo clean
    Clean,
    /// Run cargo clippy linter
    Clippy {
        #[clap(flatten)]
        locked: Locked,
        #[clap(flatten)]
        features: Features,
    },
    /// disassemble bldb
    Disasm {
        #[clap(flatten)]
        profile: BuildProfile,
        #[clap(flatten)]
        locked: Locked,
        #[clap(flatten)]
        features: Features,

        /// Interleave source and assembler output
        #[clap(long)]
        source: bool,
    },
    /// Expand macros
    Expand,
    /// Run unit tests
    Test {
        #[clap(flatten)]
        profile: BuildProfile,
        #[clap(flatten)]
        locked: Locked,
        #[clap(flatten)]
        features: Features,
    },
}

/// Mutually exclusive debug/release flags, used by all commands
/// that run builds.
#[derive(Clone, Parser)]
struct BuildProfile {
    /// Build debug version (default)
    #[clap(long, conflicts_with_all = &["release"])]
    debug: bool,

    /// Build optimized version
    #[clap(long)]
    release: bool,
}

impl BuildProfile {
    // Returns the cargo argument corresponding to the given
    // profile.
    fn to_str(&self) -> &'static str {
        self.release.then_some("--release").unwrap_or("")
    }

    // Returns the output subdirectory component corresponding
    // to the profile.
    fn dir(&self) -> &'static Path {
        Path::new(if self.release { "release" } else { "debug" })
    }
}

/// Cargo `--locked` setting; separate from BuildProfile because
/// `clippy` uses it but doesn't care about debug/release.
#[derive(Parser)]
struct Locked {
    /// Build locked to Cargo.lock
    #[clap(long)]
    locked: bool,
}

impl Locked {
    fn to_str(&self) -> &str {
        self.locked.then_some("--locked").unwrap_or("")
    }
}

/// Cargo `--features` setting.
#[derive(Parser)]
struct Features {
    #[clap(long)]
    features: Option<String>,
}

impl Features {
    // Returns the cargo argument corresponding to the given
    // features.
    fn to_string(&self) -> String {
        self.features
            .clone()
            .map(|features| format!("--features={}", features))
            .unwrap_or("".into())
    }
}

fn main() {
    let xtask = Xtask::parse();
    match xtask.cmd {
        Command::Build { profile, locked, features } => {
            build(profile, locked, features)
        }
        Command::Test { profile, locked, features } => {
            test(profile, locked, features)
        }
        Command::Disasm { profile, locked, features, source } => {
            disasm(profile, locked, features, source)
        }
        Command::Expand => expand(),
        Command::Clippy { locked, features } => clippy(locked, features),
        Command::Clean => clean(),
    }
}

/// Runs a cross-compiled build.
fn build(profile: BuildProfile, locked: Locked, features: Features) {
    let profile = profile.to_str();
    let locked = locked.to_str();
    let features = features.to_string();
    let target = target();
    let args = format!(
        "build {profile} {locked} {features} \
            -Z build-std=core,alloc \
            -Z build-std-features=compiler-builtins-mem \
            --target {target}.json"
    );
    cmd(cargo(), args.split_whitespace()).run().expect("build successful");
}

/// Runs tests.
fn test(profile: BuildProfile, locked: Locked, features: Features) {
    let profile = profile.to_str();
    let locked = locked.to_str();
    let features = features.to_string();
    let args = format!("test {profile} {locked} {features}");
    cmd(cargo(), args.split_whitespace()).run().expect("test successful");
}

/// Build and disassemble the bldb binary.
fn disasm(
    profile: BuildProfile,
    locked: Locked,
    features: Features,
    source: bool,
) {
    build(profile.clone(), locked, features);
    let triple = target();
    let profile_dir = profile.dir().to_str().unwrap();
    let flags = source.then_some("-S").unwrap_or("");
    let args = format!("-Cd {flags} target/{triple}/{profile_dir}/bldb");
    println!("args = {args}");
    cmd(objdump(), args.split_whitespace())
        .run()
        .expect("disassembly successful");
}

/// Expands macros.
fn expand() {
    cmd!(cargo(), "rustc", "--", "-Zunpretty=expanded")
        .run()
        .expect("expand successful");
}

/// Runs the Clippy linter.
fn clippy(locked: Locked, features: Features) {
    let locked = locked.to_str();
    let features = features.to_string();
    let args = format!("clippy {locked} {features}");
    cmd(cargo(), args.split_whitespace()).run().expect("clippy successful");
}

/// Runs clean on the project.
fn clean() {
    cmd!(cargo(), "clean").run().expect("clean successful");
}

/// Returns the value of the given environment variable,
/// or the default if unspecified.
fn env_or(var: &str, default: &str) -> String {
    env::var(var).unwrap_or(default.into())
}

/// Returns the name of the cargo binary.
fn cargo() -> String {
    env_or("CARGO", "cargo")
}

/// Returns the target triple we are building for.
fn target() -> String {
    env_or("TARGET", "x86_64-oxide-none-elf")
}

/// Locates the LLVM objdump binary.
fn objdump() -> String {
    env_or("OBJDUMP", "llvm-objdump".into())
}
