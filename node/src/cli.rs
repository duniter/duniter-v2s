// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

pub use duniter_cli::{DuniterConfigExtension, Sealing};

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub options: duniter_cli::RunOptions,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Run distance oracle.
    #[cfg(feature = "distance-oracle")]
    DistanceOracle(DistanceOracle),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Key management cli utilities
    #[clap(subcommand)]
    Key(crate::command::key::KeySubcommand),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sign a message, with a given (secret) key.
    Sign(sc_cli::SignCmd),

    /// Some tools for developers and advanced testers
    #[clap(subcommand)]
    Utils(crate::command::utils::UtilsSubCommand),

    /// Generate a seed that provides a vanity address.
    Vanity(sc_cli::VanityCmd),

    /// Verify a signature for a message, provided on STDIN, with a given (public or secret) key.
    Verify(sc_cli::VerifyCmd),

    /// Generate completion for various shell interpreters
    Completion(Completion),

    /// Sub-commands concerned with benchmarking.
    /// The pallet benchmarking moved to the `pallet` sub-command.
    #[cfg(feature = "runtime-benchmarks")]
    #[clap(subcommand)]
    Benchmark(Box<frame_benchmarking_cli::BenchmarkCmd>),

    /// Sub-commands concerned with benchmarking.
    #[cfg(not(feature = "runtime-benchmarks"))]
    Benchmark,
}

#[derive(Debug, clap::Args)]
pub struct Completion {
    #[clap(short, long, value_enum)]
    pub generator: clap_complete::Shell,
}

#[cfg(feature = "distance-oracle")]
#[derive(Debug, clap::Parser)]
pub struct DistanceOracle {
    /// Saving path.
    #[clap(short = 'd', long, default_value = "/tmp/duniter/chains/gdev/distance")]
    pub evaluation_result_dir: String,
    /// Number of seconds between two evaluations (oneshot if absent).
    #[clap(short = 'i', long)]
    pub interval: Option<u64>,
    /// Node used for fetching state.
    #[clap(short = 'u', long, default_value = "ws://127.0.0.1:9944")]
    pub rpc_url: String,
    /// Sets the logging level (e.g., debug, error, info, trace, warn).
    #[clap(short = 'l', long, default_value = "info")]
    pub log: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn daemon_schema_matches_the_node_parser() {
        let mut node = Cli::command();
        node.build();
        let mut shared = duniter_cli::RunOptions::command();
        shared.build();
        for arg in shared
            .get_arguments()
            .filter(|arg| arg.get_long() != Some("help"))
        {
            let actual = node
                .get_arguments()
                .find(|a| a.get_id() == arg.get_id())
                .unwrap();
            assert_eq!(actual.get_long(), arg.get_long());
            assert_eq!(actual.get_num_args(), arg.get_num_args());
            assert_eq!(actual.get_default_values(), arg.get_default_values());
        }
        let cli = Cli::try_parse_from(["duniter", "--blocks-pruning", "512"]).unwrap();
        assert!(cli.subcommand.is_none());
    }
}
