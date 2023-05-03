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

mod common;

use async_trait::async_trait;
use common::*;
use cucumber::{given, then, when, World, WorldInit};
use sp_keyring::AccountKeyring;
use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// ===== world =====

#[derive(WorldInit)]
pub struct DuniterWorld {
    ignore_errors: bool,
    inner: Option<DuniterWorldInner>,
}

impl DuniterWorld {
    // Write methods
    async fn init(&mut self, maybe_genesis_conf_file: Option<PathBuf>) {
        if let Some(ref mut inner) = self.inner {
            inner.kill();
        }
        self.inner = Some(DuniterWorldInner::new(maybe_genesis_conf_file).await);
    }
    fn kill(&mut self) {
        if let Some(ref mut inner) = self.inner {
            inner.kill();
        }
    }
    fn set_ignore_errors(&mut self, ignore_errors: bool) {
        self.ignore_errors = ignore_errors;
    }
    // Read methods
    fn client(&self) -> &Client {
        if let Some(ref inner) = self.inner {
            &inner.client
        } else {
            panic!("uninit")
        }
    }
    fn ignore_errors(&self) -> bool {
        self.ignore_errors
    }
    // Read storage entry on last block
    fn read<'a, Address>(
        &self,
        address: &'a Address,
    ) -> impl std::future::Future<
        Output = std::result::Result<
            Option<<Address::Target as subxt::metadata::DecodeWithMetadata>::Target>,
            subxt::error::Error,
        >,
    > + 'a
    where
        Address: subxt::storage::StorageAddress<IsFetchable = subxt::storage::address::Yes> + 'a,
    {
        self.client().storage().fetch(address, None)
    }
    // Read storage entry with default value (on last block)
    fn read_or_default<'a, Address>(
        &self,
        address: &'a Address,
    ) -> impl std::future::Future<
        Output = std::result::Result<
            <Address::Target as subxt::metadata::DecodeWithMetadata>::Target,
            subxt::error::Error,
        >,
    > + 'a
    where
        Address: subxt::storage::StorageAddress<
                IsFetchable = subxt::storage::address::Yes,
                IsDefaultable = subxt::storage::address::Yes,
            > + 'a,
    {
        self.client().storage().fetch_or_default(address, None)
    }
}

impl std::fmt::Debug for DuniterWorld {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[async_trait(?Send)]
impl World for DuniterWorld {
    // We do require some error type.
    type Error = Infallible;

    async fn new() -> std::result::Result<Self, Infallible> {
        Ok(Self {
            ignore_errors: false,
            inner: None,
        })
    }
}

struct DuniterWorldInner {
    client: Client,
    process: Process,
}

impl DuniterWorldInner {
    async fn new(maybe_genesis_conf_file: Option<PathBuf>) -> Self {
        let (client, process) = spawn_node(maybe_genesis_conf_file).await;
        DuniterWorldInner { client, process }
    }
    fn kill(&mut self) {
        self.process.kill();
    }
}

fn parse_amount(amount: u64, unit: &str) -> (u64, bool) {
    match unit {
        "ĞD" => (amount * 100, false),
        "cĞD" => (amount, false),
        "UD" => (amount * 1_000, true),
        "mUD" => (amount, true),
        _ => unreachable!(),
    }
}

// ===== given =====

#[given(regex = r"([a-zA-Z]+) ha(?:ve|s) (\d+) (ĞD|cĞD|UD|mUD)")]
async fn who_have(world: &mut DuniterWorld, who: String, amount: u64, unit: String) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who).expect("unknown to");
    let (mut amount, is_ud) = parse_amount(amount, &unit);

    if is_ud {
        let current_ud_amount = world
            .read(&gdev::storage().universal_dividend().current_ud())
            .await?
            .unwrap_or_default();
        amount = (amount * current_ud_amount) / 1_000;
    }

    // Create {amount} ĞD for {who}
    common::balances::set_balance(world.client(), who, amount).await?;

    Ok(())
}

// ===== when =====

#[when(regex = r"(\d+) blocks? later")]
async fn n_blocks_later(world: &mut DuniterWorld, n: usize) -> Result<()> {
    for _ in 0..n {
        common::create_empty_block(world.client()).await?;
    }
    Ok(())
}

#[when(regex = r"([a-zA-Z]+) sends? (\d+) (ĞD|cĞD|UD|mUD) to ([a-zA-Z]+)$")]
async fn transfer(
    world: &mut DuniterWorld,
    from: String,
    amount: u64,
    unit: String,
    to: String,
) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");
    let (amount, is_ud) = parse_amount(amount, &unit);

    let res = if is_ud {
        common::balances::transfer_ud(world.client(), from, amount, to).await
    } else {
        common::balances::transfer(world.client(), from, amount, to).await
    };

    if world.ignore_errors() {
        Ok(())
    } else {
        res
    }
}

#[when(regex = r"([a-zA-Z]+) sends? (\d+) (ĞD|cĞD) to oneshot ([a-zA-Z]+)")]
async fn create_oneshot_account(
    world: &mut DuniterWorld,
    from: String,
    amount: u64,
    unit: String,
    to: String,
) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");
    let (amount, is_ud) = parse_amount(amount, &unit);

    assert!(!is_ud);

    common::oneshot::create_oneshot_account(world.client(), from, amount, to).await
}

#[when(regex = r"oneshot ([a-zA-Z]+) consumes? into (oneshot|account) ([a-zA-Z]+)")]
async fn consume_oneshot_account(
    world: &mut DuniterWorld,
    from: String,
    is_dest_oneshot: String,
    to: String,
) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");
    let to = match is_dest_oneshot.as_str() {
        "oneshot" => common::oneshot::Account::Oneshot(to),
        "account" => common::oneshot::Account::Normal(to),
        _ => unreachable!(),
    };

    common::oneshot::consume_oneshot_account(world.client(), from, to).await
}

#[when(
    regex = r"oneshot ([a-zA-Z]+) consumes? (\d+) (ĞD|cĞD) into (oneshot|account) ([a-zA-Z]+) and the rest into (oneshot|account) ([a-zA-Z]+)"
)]
#[allow(clippy::too_many_arguments)]
async fn consume_oneshot_account_with_remaining(
    world: &mut DuniterWorld,
    from: String,
    amount: u64,
    unit: String,
    is_dest_oneshot: String,
    to: String,
    is_remaining_to_oneshot: String,
    remaining_to: String,
) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");
    let remaining_to = AccountKeyring::from_str(&remaining_to).expect("unknown remaining_to");
    let to = match is_dest_oneshot.as_str() {
        "oneshot" => common::oneshot::Account::Oneshot(to),
        "account" => common::oneshot::Account::Normal(to),
        _ => unreachable!(),
    };
    let remaining_to = match is_remaining_to_oneshot.as_str() {
        "oneshot" => common::oneshot::Account::Oneshot(remaining_to),
        "account" => common::oneshot::Account::Normal(remaining_to),
        _ => unreachable!(),
    };
    let (amount, is_ud) = parse_amount(amount, &unit);

    assert!(!is_ud);

    common::oneshot::consume_oneshot_account_with_remaining(
        world.client(),
        from,
        amount,
        to,
        remaining_to,
    )
    .await
}

#[when(regex = r"([a-zA-Z]+) sends? all (?:his|her) (?:ĞDs?|DUs?|UDs?) to ([a-zA-Z]+)")]
async fn send_all_to(world: &mut DuniterWorld, from: String, to: String) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");

    common::balances::transfer_all(world.client(), from, to).await
}

#[when(regex = r"([a-zA-Z]+) certifies ([a-zA-Z]+)")]
async fn certifies(world: &mut DuniterWorld, from: String, to: String) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");

    common::cert::certify(world.client(), from, to).await
}

#[when(regex = r"([a-zA-Z]+) creates identity for ([a-zA-Z]+)")]
async fn creates_identity(world: &mut DuniterWorld, from: String, to: String) -> Result<()> {
    // Parse inputs
    let from = AccountKeyring::from_str(&from).expect("unknown from");
    let to = AccountKeyring::from_str(&to).expect("unknown to");

    common::identity::create_identity(world.client(), from, to).await
}

#[when(regex = r#"([a-zA-Z]+) confirms (?:his|her) identity with pseudo "([a-zA-Z]+)""#)]
async fn confirm_identity(world: &mut DuniterWorld, from: String, pseudo: String) -> Result<()> {
    let from = AccountKeyring::from_str(&from).expect("unknown from");

    common::identity::confirm_identity(world.client(), from, pseudo).await
}

// ===== then ====

#[then(regex = r"([a-zA-Z]+) should have (\d+) (ĞD|cĞD)")]
async fn should_have(
    world: &mut DuniterWorld,
    who: String,
    amount: u64,
    unit: String,
) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who)
        .expect("unknown to")
        .to_account_id();
    let (amount, _is_ud) = parse_amount(amount, &unit);

    let who_account = world
        .read_or_default(&gdev::storage().system().account(&who))
        .await?;
    assert_eq!(who_account.data.free, amount);
    Ok(())
}

#[then(regex = r"([a-zA-Z]+) should have oneshot (\d+) (ĞD|cĞD)")]
async fn should_have_oneshot(
    world: &mut DuniterWorld,
    who: String,
    amount: u64,
    unit: String,
) -> Result<()> {
    // Parse inputs
    let who = AccountKeyring::from_str(&who)
        .expect("unknown to")
        .to_account_id();
    let (amount, _is_ud) = parse_amount(amount, &unit);

    let oneshot_amount = world
        .read(&gdev::storage().oneshot_account().oneshot_accounts(&who))
        .await?;
    assert_eq!(oneshot_amount.unwrap_or(0), amount);
    Ok(())
}

#[then(regex = r"Current UD amount should be (\d+).(\d+)")]
async fn current_ud_amount_should_be(
    world: &mut DuniterWorld,
    amount: u64,
    cents: u64,
) -> Result<()> {
    let expected = (amount * 100) + cents;
    let actual = world
        .read_or_default(&gdev::storage().universal_dividend().current_ud())
        .await?;
    assert_eq!(actual, expected);
    Ok(())
}

#[then(regex = r"Monetary mass should be (\d+).(\d+)")]
async fn monetary_mass_should_be(world: &mut DuniterWorld, amount: u64, cents: u64) -> Result<()> {
    let expected = (amount * 100) + cents;
    let actual = world
        .read_or_default(&gdev::storage().universal_dividend().monetary_mass())
        .await?;
    assert_eq!(actual, expected);
    Ok(())
}

#[then(regex = r"([a-zA-Z]+) should be certified by ([a-zA-Z]+)")]
async fn should_be_certified_by(
    world: &mut DuniterWorld,
    receiver: String,
    issuer: String,
) -> Result<()> {
    // Parse inputs
    let receiver_account = AccountKeyring::from_str(&receiver)
        .expect("unknown to")
        .to_account_id();
    let issuer_account = AccountKeyring::from_str(&issuer)
        .expect("unknown to")
        .to_account_id();

    // get corresponding identities index
    let issuer_index = world
        .read(
            &gdev::storage()
                .identity()
                .identity_index_of(&issuer_account),
        )
        .await?
        .unwrap();
    let receiver_index = world
        .read(
            &gdev::storage()
                .identity()
                .identity_index_of(&receiver_account),
        )
        .await?
        .unwrap();

    let issuers = world
        .read_or_default(&gdev::storage().cert().certs_by_receiver(receiver_index))
        .await?;

    // look for certification by issuer/receiver pair
    match issuers.binary_search_by(|(issuer_, _)| issuer_.cmp(&issuer_index)) {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!(
            "no certification found from {} to {}: {:?}",
            issuer,
            receiver,
            issuers
        )
        .into()),
    }
}

use gdev::runtime_types::pallet_identity::types::IdtyStatus;

#[then(regex = r"([a-zA-Z]+) identity should be created")]
async fn identity_should_be_created(world: &mut DuniterWorld, receiver: String) -> Result<()> {
    let identity_value = common::identity::get_identity_value(world, receiver).await?;

    match identity_value.status {
        IdtyStatus::Created => Ok(()),
        IdtyStatus::ConfirmedByOwner | IdtyStatus::Validated => {
            Err(anyhow::anyhow!("status not created").into())
        }
    }
}

#[then(regex = r"([a-zA-Z]+) identity should be confirmed")]
async fn identity_should_be_confirmed(world: &mut DuniterWorld, name: String) -> Result<()> {
    let identity_value = common::identity::get_identity_value(world, name).await?;

    match identity_value.status {
        IdtyStatus::ConfirmedByOwner => Ok(()),
        IdtyStatus::Created | IdtyStatus::Validated => {
            Err(anyhow::anyhow!("status not confirmed by owner").into())
        }
    }
}

// ============================================================

#[derive(clap::Args)]
struct CustomOpts {
    /// Keep running
    #[clap(short, long)]
    keep_running: bool,
}

const DOCKER_FEATURES_PATH: &str = "/var/lib/duniter/cucumber-features";
const LOCAL_FEATURES_PATH: &str = "cucumber-features";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    //env_logger::init();

    let features_path = if std::path::Path::new(DOCKER_FEATURES_PATH).exists() {
        DOCKER_FEATURES_PATH
    } else if std::path::Path::new(LOCAL_FEATURES_PATH).exists() {
        LOCAL_FEATURES_PATH
    } else {
        panic!("cucumber-features folder not found");
    };

    let opts = cucumber::cli::Opts::<_, _, _, CustomOpts>::parsed();
    let keep_running = opts.custom.keep_running;

    // Handle crtl+C
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    DuniterWorld::cucumber()
        //.fail_on_skipped()
        .max_concurrent_scenarios(4)
        .before(|feature, _rule, scenario, world| {
            let mut genesis_conf_file_path = PathBuf::new();
            genesis_conf_file_path.push("cucumber-genesis");
            genesis_conf_file_path.push(&format!(
                "{}.json",
                genesis_conf_name(&feature.tags, &scenario.tags)
            ));
            world.set_ignore_errors(ignore_errors(&scenario.tags));
            Box::pin(world.init(Some(genesis_conf_file_path)))
        })
        .after(move |_feature, _rule, _scenario, maybe_world| {
            if keep_running {
                while running.load(Ordering::SeqCst) {}
            }

            if let Some(world) = maybe_world {
                world.kill();
            }
            Box::pin(std::future::ready(()))
        })
        .with_cli(opts)
        .run_and_exit(features_path)
        .await;
}

fn genesis_conf_name(feature_tags: &[String], scenario_tags: &[String]) -> String {
    for tag in scenario_tags {
        if let Some(("genesis", conf_name)) = tag.split_once('.') {
            return conf_name.to_owned();
        }
    }
    for tag in feature_tags {
        if let Some(("genesis", conf_name)) = tag.split_once('.') {
            return conf_name.to_owned();
        }
    }
    "default".to_owned()
}

fn ignore_errors(scenario_tags: &[String]) -> bool {
    for tag in scenario_tags {
        if tag == "ignoreErrors" {
            return true;
        }
    }
    false
}
