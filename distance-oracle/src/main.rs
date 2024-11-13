// Copyright 2023-2024 Axiom-Team
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

use clap::Parser;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(short = 'd', long, default_value = "/tmp/duniter/chains/gdev/distance")]
    evaluation_result_dir: String,
    /// Number of seconds between two evaluations (oneshot if absent)
    #[clap(short = 'i', long, default_value = "None")]
    interval: Option<u64>,
    /// Node used for fetching state
    #[clap(short = 'u', long, default_value = "ws://127.0.0.1:9944")]
    rpc_url: String,
    /// Log level (off, error, warn, info, debug, trace)
    #[clap(short = 'l', long, default_value = "info")]
    log: log::LevelFilter,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    simple_logger::SimpleLogger::new()
        .with_level(cli.log)
        .init()
        .unwrap();

    let client = distance_oracle::api::client(&cli.rpc_url).await;

    let settings = distance_oracle::Settings {
        evaluation_result_dir: cli.evaluation_result_dir.into(),
        rpc_url: cli.rpc_url,
    };

    if let Some(duration) = cli.interval {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(duration));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            distance_oracle::run_and_save(&client, &settings).await;
            interval.tick().await;
        }
    } else {
        distance_oracle::run_and_save(&client, &settings).await;
    }
}
