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

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// substrate base options
    #[clap(flatten)]
    pub run: sc_cli::RunCmd,

    /// duniter specific options
    #[clap(flatten)]
    pub duniter_options: DuniterConfigExtension,

    /// How blocks should be sealed
    ///
    /// Options are "production", "instant", "manual", or timer interval in milliseconds
    #[clap(long, default_value = "production")]
    pub sealing: crate::cli::Sealing,
}

/// add options specific to duniter client
#[derive(Debug, Default, Clone, clap::Parser)]
pub struct DuniterConfigExtension {
    /// Public RPC endpoint to gossip on the network and make available in the apps.
    #[arg(long)]
    pub public_rpc: Option<String>,

    /// Public Squid graphql endpoint to gossip on the network and make available in the apps. Convention: `<domain.tld>/v1/graphql`
    #[arg(long)]
    pub public_squid: Option<String>,

    /// Public endpoints from a JSON file, using following format where `protocol` and `address` are
    /// strings (value is free) :
    ///
    /// ```json
    /// {
    ///     "endpoints": [
    ///         { "protocol": "rpc", "address": "wss://gdev.example.com" },
    ///         { "protocol": "squid", "address": "gdev.example.com/v1/graphql" },
    ///         { "protocol": "other", "address": "gdev.example.com/other" }
    ///     ]
    /// }
    /// ```
    #[arg(long, value_name = "JSON_FILE_PATH")]
    pub public_endpoints: Option<String>,
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
    #[clap(subcommand)]
    Benchmark(Box<frame_benchmarking_cli::BenchmarkCmd>),
}

/// Block authoring scheme to be used by the node
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Sealing {
    /// Author a block using normal runtime behavior (mandatory for production networks)
    Production,
    /// Author a block immediately upon receiving a transaction into the transaction pool
    Instant,
    /// Author a block upon receiving an RPC command
    Manual,
    /// Author blocks at a regular interval specified in milliseconds
    // Clap limitiation with non-unit variant.
    // While it compiles just fine with clap alone, clap_complete emits a compile-time error.
    // See https://github.com/clap-rs/clap/issues/3543
    #[clap(skip)]
    Interval(u64),
}

impl Sealing {
    pub fn is_manual_consensus(self) -> bool {
        self != Self::Production
    }
}

impl std::str::FromStr for Sealing {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "production" => Self::Production,
            "instant" => Self::Instant,
            "manual" => Self::Manual,
            s => {
                let millis = s
                    .parse::<u64>()
                    .map_err(|_| "couldn't decode sealing param")?;
                Self::Interval(millis)
            }
        })
    }
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
