use lc_core_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, IdentityConfig, IdtyDid,
    IdtyIndex, IdtyRight, IdtyValue, Planet, Signature, StrongCertConfig, SudoConfig, SystemConfig,
    UdAccountsStorageConfig, UniversalDividendConfig, WASM_BINARY,
};
use maplit::btreemap;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::{BTreeMap, BTreeSet};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

const TOKEN_DECIMALS: usize = 2;
const TOKEN_SYMBOL: &str = "ĞT";
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Create a fake did (for dev and testnet)
fn did(u8_: u8) -> IdtyDid {
    IdtyDid {
        hash: sp_core::H256::repeat_byte(u8_),
        planet: Planet::Earth,
        latitude: 0,
        longitude: 0,
    }
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Inital identities
                btreemap![
                    did(1) => get_account_id_from_seed::<sr25519::Public>("Alice"),
                    did(2) => get_account_id_from_seed::<sr25519::Public>("Bob"),
                    did(3) => get_account_id_from_seed::<sr25519::Public>("Charlie"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(
            serde_json::json!({
                    "tokenDecimals": TOKEN_DECIMALS,
                    "tokenSymbol": TOKEN_SYMBOL,
            })
            .as_object()
            .expect("must be a map")
            .clone(),
        ),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Initial identities
                btreemap![
                    did(1) => get_account_id_from_seed::<sr25519::Public>("Alice"),
                    did(2) => get_account_id_from_seed::<sr25519::Public>("Bob"),
                    did(3) => get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    did(4) => get_account_id_from_seed::<sr25519::Public>("Dave"),
                    did(5) => get_account_id_from_seed::<sr25519::Public>("Eve"),
                    did(6) => get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(
            serde_json::json!({
                    "tokenDecimals": TOKEN_DECIMALS,
                    "tokenSymbol": TOKEN_SYMBOL,
            })
            .as_object()
            .expect("must be a map")
            .clone(),
        ),
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    initial_identities: BTreeMap<IdtyDid, AccountId>,
    root_key: AccountId,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of INITIAL_BALANCE.
            balances: Vec::with_capacity(0),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        identity: IdentityConfig {
            identities: initial_identities
                .iter()
                .map(|(did, account)| IdtyValue {
                    did: *did,
                    expire_on: lc_core_runtime::MaxInactivityPeriod::get(),
                    owner_key: account.clone(),
                    removable_on: 0,
                    renewable_on: lc_core_runtime::StrongCertRenewablePeriod::get(),
                    rights: vec![
                        (IdtyRight::CreateIdty, None),
                        (IdtyRight::StrongCert, None),
                        (IdtyRight::Ud, None),
                    ],
                    status: lc_core_runtime::IdtyStatus::Validated,
                    data: Default::default(),
                })
                .collect(),
        },
        strong_cert: StrongCertConfig {
            certs_by_issuer: clique_wot(initial_identities.len()),
            phantom: std::marker::PhantomData,
        },
        ud_accounts_storage: UdAccountsStorageConfig {
            ud_accounts: initial_identities.values().cloned().collect(),
        },
        universal_dividend: UniversalDividendConfig {
            first_ud: 1_000,
            initial_monetary_mass: 0,
        },
    }
}

fn clique_wot(initial_identities_len: usize) -> BTreeMap<IdtyIndex, BTreeSet<IdtyIndex>> {
    let mut certs_by_issuer = BTreeMap::new();
    for i in 1..=initial_identities_len {
        certs_by_issuer.insert(
            i as IdtyIndex,
            (1..=initial_identities_len)
                .filter_map(|j| if i != j { Some(j as IdtyIndex) } else { None })
                .collect(),
        );
    }
    certs_by_issuer
}
