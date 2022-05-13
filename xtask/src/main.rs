// Copyright 2021 Axiom-Team
//
// This file is part of Substrate-Libre-Currency.
//
// Substrate-Libre-Currency is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Substrate-Libre-Currency is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Substrate-Libre-Currency. If not, see <https://www.gnu.org/licenses/>.

mod gen_calls_doc;

use anyhow::Result;
use clap::Parser;
use std::process::Command;

const MIN_RUST_VERSION: &str = "1.58.0";

#[derive(Debug, clap::Parser)]
struct DuniterXTask {
    #[clap(subcommand)]
    command: DuniterXTaskCommand,
}

#[derive(Debug, clap::Subcommand)]
enum DuniterXTaskCommand {
    /// Build duniter binary
    Build {
        #[clap(long)]
        production: bool,
    },
    /// Generate calls documentation
    GenCallsDoc,
    /// Execute unit tests and integration tests
    /// End2tests are skipped
    Test,
}

fn main() -> Result<()> {
    let args = DuniterXTask::parse();

    if !version_check::is_min_version(MIN_RUST_VERSION).unwrap_or(false)
        && exec_should_success(Command::new("rustup").args(&["update", "stable"])).is_err()
    {
        eprintln!(
                "Duniter requires stable Rust {} or higher. If you installed the Rust toolchain via rustup, please execute the command `rustup update stable`.",
                MIN_RUST_VERSION
            );
        std::process::exit(1);
    }
    Command::new("rustc").arg("--version").status()?;
    Command::new("cargo").arg("--version").status()?;

    match args.command {
        DuniterXTaskCommand::Build { production } => build(production),
        DuniterXTaskCommand::GenCallsDoc => gen_calls_doc::gen_calls_doc(),
        DuniterXTaskCommand::Test => test(),
    }
}

fn build(_production: bool) -> Result<()> {
    exec_should_success(Command::new("cargo").args(&["clean", "-p", "duniter"]))?;
    exec_should_success(Command::new("cargo").args(&["build", "--locked"]))?;
    exec_should_success(Command::new("mkdir").args(&["build"]))?;
    exec_should_success(Command::new("mv").args(&["target/debug/duniter", "build/duniter"]))?;

    Ok(())
}

fn test() -> Result<()> {
    exec_should_success(Command::new("cargo").args(&[
        "test",
        "--workspace",
        "--exclude",
        "duniter-end2end-tests",
    ]))?;

    Ok(())
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
