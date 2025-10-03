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

use anyhow::{Context, Result, bail};
use codec::Decode;
use core::hash::Hash;
use frame_metadata::v16::{StorageEntryModifier, StorageEntryType};
use scale_info::{PortableRegistry, Type, TypeDef, form::PortableForm};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Command,
};
use tera::Tera;
use weightanalyzer::{MaxBlockWeight, WeightInfo, analyze_weight};

fn rename_key<K, V>(h: &mut HashMap<K, V>, old_key: &K, new_key: K)
where
    K: Eq + Hash,
{
    if let Some(v) = h.remove(old_key) {
        h.insert(new_key, v);
    }
}

// consts

const CALLS_DOC_FILEPATH: &str = "docs/api/runtime-calls.md";
const EVENTS_DOC_FILEPATH: &str = "docs/api/runtime-events.md";
const STORAGES_DOC_FILEPATH: &str = "docs/api/runtime-storages.md";
const CONSTANTS_DOC_FILEPATH: &str = "docs/api/runtime-constants.md";
const ERRORS_DOC_FILEPATH: &str = "docs/api/runtime-errors.md";
const ERRORS_PO_FILEPATH: &str = "docs/api/runtime-errors.po";
const TEMPLATES_GLOB: &str = "xtask/res/templates/*.{md,po}";
const WEIGHT_FILEPATH: &str = "runtime/gdev/src/weights/";

// define structs and implementations

type RuntimePallets = Vec<Pallet>;

#[derive(Clone, Serialize)]
struct Pallet {
    index: u8,
    name: String,
    type_name: String,
    calls: Vec<Call>,
    events: Vec<Event>,
    errors: Vec<ErroR>,
    storages: Vec<Storage>,
    constants: Vec<Constant>,
}
#[derive(Clone, Serialize)]
struct Call {
    documentation: String,
    index: u8,
    name: String,
    params: Vec<CallParam>,
    weight: f64,
}
#[derive(Clone, Serialize)]
struct CallParam {
    name: String,
    type_name: String,
}
#[derive(Clone, Serialize)]
struct Event {
    documentation: String,
    index: u8,
    name: String,
    params: Vec<EventParam>,
}
#[derive(Clone, Serialize)]
struct EventParam {
    name: String,
    type_name: String,
}
#[derive(Clone, Serialize)]
struct ErroR {
    documentation: String,
    index: u8,
    name: String,
}
#[derive(Clone, Serialize)]
struct Storage {
    documentation: String,
    name: String,
    type_key: String,
    type_value: String,
}
#[derive(Clone, Serialize)]
struct Constant {
    documentation: String,
    name: String,
    value: String,
    type_value: String,
}

impl Pallet {
    #![allow(clippy::too_many_arguments)]
    fn new(
        index: u8,
        name: String,
        type_name: String,
        call_scale_type_def: &Option<scale_info::TypeDef<PortableForm>>,
        event_scale_type_def: &Option<scale_info::TypeDef<PortableForm>>,
        error_scale_type_def: &Option<scale_info::TypeDef<PortableForm>>,
        storages: Vec<Storage>,
        constants: Vec<Constant>,
    ) -> Result<Self> {
        let calls = if let Some(call_scale_type_def) = call_scale_type_def {
            if let scale_info::TypeDef::Variant(calls_enum) = call_scale_type_def {
                calls_enum.variants.iter().map(Into::into).collect()
            } else {
                bail!("Invalid metadata")
            }
        } else {
            vec![]
        };
        let events = if let Some(event_scale_type_def) = event_scale_type_def {
            if let scale_info::TypeDef::Variant(events_enum) = event_scale_type_def {
                events_enum.variants.iter().map(Into::into).collect()
            } else {
                bail!("Invalid metadata")
            }
        } else {
            vec![]
        };
        let errors = if let Some(error_scale_type_def) = error_scale_type_def {
            if let scale_info::TypeDef::Variant(errors_enum) = error_scale_type_def {
                errors_enum.variants.iter().map(Into::into).collect()
            } else {
                bail!("Invalid metadata")
            }
        } else {
            vec![]
        };

        Ok(Self {
            index,
            name,
            type_name,
            calls,
            events,
            errors,
            storages,
            constants,
        })
    }
}

impl From<&scale_info::Variant<PortableForm>> for Call {
    fn from(variant: &scale_info::Variant<PortableForm>) -> Self {
        Self {
            documentation: variant
                .docs
                .iter()
                .take_while(|line| !line.starts_with("# <weight>"))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n"),
            index: variant.index,
            name: variant.name.to_owned(),
            params: variant.fields.iter().map(Into::into).collect(),
            weight: Default::default(),
        }
    }
}
impl From<&scale_info::Field<PortableForm>> for CallParam {
    fn from(field: &scale_info::Field<PortableForm>) -> Self {
        Self {
            name: field.clone().name.unwrap_or_default().to_string(),
            type_name: field.clone().type_name.unwrap_or_default().to_string(),
        }
    }
}

impl From<&scale_info::Variant<PortableForm>> for Event {
    fn from(variant: &scale_info::Variant<PortableForm>) -> Self {
        Self {
            documentation: variant.docs.to_vec().join("\n"),
            index: variant.index,
            name: variant.name.to_owned(),
            params: variant.fields.iter().map(Into::into).collect(),
        }
    }
}
impl From<&scale_info::Field<PortableForm>> for EventParam {
    fn from(field: &scale_info::Field<PortableForm>) -> Self {
        Self {
            name: field.clone().name.unwrap_or_default().to_string(),
            type_name: field.clone().type_name.unwrap_or_default().to_string(),
        }
    }
}

impl From<&scale_info::Variant<PortableForm>> for ErroR {
    fn from(variant: &scale_info::Variant<PortableForm>) -> Self {
        Self {
            documentation: variant.docs.to_vec().join("\n"),
            index: variant.index,
            name: variant.name.to_owned(),
        }
    }
}

// classify calls into categories depending on their origin
enum CallCategory {
    // calls filtered by runtime
    Disabled,
    // inherents
    Inherent,
    // ensure_root
    Root,
    // sudo
    Sudo,
    // user calls
    User,
    // other (like a certain proportion of technical comittee)
    OtherOrigin,
}

impl CallCategory {
    fn is(pallet_name: &str, call_name: &str) -> Self {
        match (pallet_name, call_name) {
            // substrate "system"
            ("System", _) => Self::Root,
            ("Scheduler", _) => Self::Root,
            ("Babe", "report_equivocation" | "report_equivocation_unsigned") => Self::Inherent,
            ("Babe", "plan_config_change") => Self::Root,
            ("Authorship", _) => Self::Inherent,
            ("Session", _) => Self::Disabled,
            ("Grandpa", "report_equivocation" | "report_equivocation_unsigned") => Self::Inherent,
            ("Grandpa", "note_stalled") => Self::Root,
            ("Timestamp", _) => Self::Inherent,
            ("ImOnline", _) => Self::Inherent,
            // substrate "common"
            (
                "Balances",
                "force_set_balance"
                | "force_transfer"
                | "force_unreserve"
                | "force_adjust_total_issuance",
            ) => Self::Root,
            ("Balances", "burn") => Self::Disabled,
            ("Sudo", _) => Self::Sudo,
            ("Treasury", "approve_proposal" | "reject_proposal") => Self::OtherOrigin,
            ("Utility", "dispatch_as" | "with_weight") => Self::Root,
            // duniter
            ("Distance", "force_update_evaluation" | "force_valid_distance_status") => Self::Root,
            ("Distance", "update_evaluation") => Self::Inherent,
            ("AuthorityMembers", "remove_member_from_blacklist" | "remove_member") => Self::Root,
            ("UpgradeOrigin", "dispatch_as_root" | "dispatch_as_root_unchecked_weight") => {
                Self::OtherOrigin
            }
            ("Identity", "remove_identity" | "prune_item_identities_names" | "fix_sufficients") => {
                Self::Root
            }
            ("Certification", "del_cert" | "remove_all_certs_received_by") => Self::Root,
            ("TechnicalCommittee", "set_members" | "disapprove_proposal") => Self::Root,
            // if not classified, consider it at a user call
            _ => Self::User,
        }
    }

    // only user calls
    fn is_user(pallet_name: &str, call_name: &str) -> bool {
        matches!(Self::is(pallet_name, call_name), Self::User)
    }
}

/// generate runtime calls documentation
pub(super) fn gen_doc() -> Result<()> {
    // Read metadata
    let mut file = std::fs::File::open("resources/gdev_metadata.scale")
        .with_context(|| "Failed to open metadata file")?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .with_context(|| "Failed to read metadata file")?;

    let metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &bytes[..])
        .with_context(|| "Failed to decode metadata")?;

    println!("Metadata successfully loaded!");

    let (mut runtime, max_weight) =
        if let frame_metadata::RuntimeMetadata::V16(ref metadata_v16) = metadata.1 {
            (
                get_from_metadata_v16(metadata_v16.clone())?,
                get_max_weight_from_metadata_v16(metadata_v16.clone())?,
            )
        } else {
            bail!("unsuported metadata version")
        };

    let mut weights = get_weights(max_weight)?;

    // Ad hoc names conversion between pallet filename and instance name
    rename_key(&mut weights, &"FrameSystem".into(), "System".into());
    rename_key(&mut weights, &"DuniterAccount".into(), "Account".into());
    rename_key(
        &mut weights,
        &"Collective".into(),
        "TechnicalCommittee".into(),
    );

    // We enforce weight for each pallet.
    // For pallets with manual or no weight, we define a default value.
    weights.insert("Babe".to_string(), Default::default()); // Manual
    weights.insert("Grandpa".to_string(), Default::default()); // Manual
    weights.insert("AtomicSwap".to_string(), Default::default()); // No weight

    // Insert weights for each call of each pallet.
    // If no weight is available, the weight is set to -1.
    // We use the relative weight in percent computed as the extrinsic base +
    // the extrinsic execution divided by the total weight available in
    // one block. If the weight depends on a complexity parameter,
    // we display the worst possible weight, taking the upper limit as
    // defined during the benchmark.
    runtime.iter_mut().for_each(|pallet| {
        pallet.calls.iter_mut().for_each(|call| {
            call.weight = weights
                .get(&pallet.name)
                .unwrap_or_else(|| panic!("{}", ("No weight for ".to_owned() + &pallet.name)))
                .get(&call.name)
                .map_or(-1f64, |weight| {
                    (weight.relative_weight * 10000.).round() / 10000.
                })
        })
    });

    let (call_doc, event_doc, error_doc, error_po, storage_doc, constant_doc) =
        print_runtime(runtime);

    // Generate docs from rust code
    Command::new("cargo")
        .args([
            "doc",
            "--package=duniter",
            "--package=pallet-*",
            "--package=*-runtime",
            "--package=*distance*",
            "--package=*membership*",
            "--no-deps",
            "--document-private-items",
            "--features=runtime-benchmarks",
            "--package=pallet-atomic-swap",
            "--package=pallet-authority-discovery",
            "--package=pallet-balances",
            "--package=pallet-collective",
            "--package=pallet-im-online",
            "--package=pallet-preimage",
            "--package=pallet-proxy",
            "--package=pallet-scheduler",
            "--package=pallet-session",
            "--package=pallet-sudo",
            "--package=pallet-timestamp",
            "--package=pallet-treasury",
            "--package=pallet-utility",
        ])
        .status()
        .expect("cargo doc failed to execute");

    let mut file = File::create(CALLS_DOC_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", CALLS_DOC_FILEPATH))?;
    file.write_all(call_doc.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", CALLS_DOC_FILEPATH))?;
    let mut file = File::create(EVENTS_DOC_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", EVENTS_DOC_FILEPATH))?;
    file.write_all(event_doc.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", EVENTS_DOC_FILEPATH))?;
    let mut file = File::create(ERRORS_DOC_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", ERRORS_DOC_FILEPATH))?;
    file.write_all(error_doc.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", ERRORS_DOC_FILEPATH))?;
    let mut file = File::create(STORAGES_DOC_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", STORAGES_DOC_FILEPATH))?;
    file.write_all(storage_doc.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", STORAGES_DOC_FILEPATH))?;
    let mut file = File::create(CONSTANTS_DOC_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", CONSTANTS_DOC_FILEPATH))?;
    file.write_all(constant_doc.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", CONSTANTS_DOC_FILEPATH))?;
    let mut file = File::create(ERRORS_PO_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", ERRORS_PO_FILEPATH))?;
    file.write_all(error_po.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", ERRORS_PO_FILEPATH))?;

    Ok(())
}

fn get_max_weight_from_metadata_v16(
    metadata_v16: frame_metadata::v16::RuntimeMetadataV16,
) -> Result<u128> {
    // Extract the maximal weight available in one block
    // from the metadata.
    let block_weights = metadata_v16
        .pallets
        .iter()
        .find(|pallet| pallet.name == "System")
        .expect("Can't find System pallet metadata")
        .constants
        .iter()
        .find(|constant| constant.name == "BlockWeights")
        .expect("Can't find BlockWeights");

    let block_weights = scale_value::scale::decode_as_type(
        &mut &*block_weights.value,
        &block_weights.ty.id,
        &metadata_v16.types,
    )
    .expect("Can't decode max_weight")
    .value;

    if let scale_value::ValueDef::Composite(scale_value::Composite::Named(i)) = block_weights
        && let scale_value::ValueDef::Composite(scale_value::Composite::Named(j)) =
            &i.iter().find(|name| name.0 == "max_block").unwrap().1.value
        && let scale_value::ValueDef::Primitive(scale_value::Primitive::U128(k)) =
            &j.iter().find(|name| name.0 == "ref_time").unwrap().1.value
    {
        Ok(*k)
    } else {
        bail!("Invalid max_weight")
    }
}

/// Converts a `Type<PortableForm>` into a human-readable string representation.
///
/// This function is a core part of the type resolution pipeline, working together with:
/// - `resolve_type()` to obtain type definitions.
/// - `format_generics()` to correctly format generic parameters.
///
/// It processes a `Type<PortableForm>` from the metadata registry and outputs a Rust-like string representation,
/// handling all supported type definitions (composite types, sequences, arrays, tuples, primitives, etc.).
///
/// # Returns
/// - `Ok(String)`: A formatted type string (e.g., `"Vec<u32>"`, `"(i32, bool)"`).
/// - `Err(anyhow::Error)`: If metadata are incorrect.
///
/// # How It Works With Other Functions:
/// - Calls `format_generics()` to handle generic type parameters.
/// - Calls `resolve_type()` when resolving inner types inside sequences, arrays, and tuples.
/// - Used by `resolve_type()` as a formatting step after retrieving a type.
///
fn format_type(ty: &Type<PortableForm>, types: &PortableRegistry) -> Result<String> {
    let path = ty.path.to_string();

    match &ty.type_def {
        TypeDef::Composite(_) => {
            let generics = format_generics(&ty.type_params, types)?;
            Ok(format!("{}{}", path, generics))
        }
        TypeDef::Variant(_) => {
            let generics = format_generics(&ty.type_params, types)?;
            Ok(format!("{}{}", path, generics))
        }
        TypeDef::Sequence(seq) => {
            let element_type = resolve_type(seq.type_param.id, types)?;
            Ok(format!("Vec<{}>", element_type))
        }
        TypeDef::Array(arr) => {
            let element_type = resolve_type(arr.type_param.id, types)?;
            Ok(format!("[{}; {}]", element_type, arr.len))
        }
        TypeDef::Tuple(tuple) => {
            let elements = tuple
                .fields
                .iter()
                .map(|f| resolve_type(f.id, types))
                .collect::<Result<Vec<String>>>()?;
            Ok(format!("({})", elements.join(", ")))
        }
        TypeDef::Primitive(primitive) => Ok(format!("{:?}", primitive)),
        TypeDef::Compact(compact) => {
            let inner_type = resolve_type(compact.type_param.id, types)?;
            Ok(format!("Compact<{}>", inner_type))
        }
        TypeDef::BitSequence(_) => Ok(String::default()),
    }
}

/// Resolves a type ID to a formatted string representation.
///
/// This function serves as a bridge between raw type IDs and fully formatted types.
/// It works closely with `format_type()`, ensuring that once a type is found, it is properly formatted.
///
/// # How It Works With Other Functions:
/// - Retrieves a type from the registry using `types.resolve(type_id)`.
/// - If successful, calls `format_type()` to get a human-readable format.
/// - Used internally by `format_type()` when resolving type dependencies.
///
fn resolve_type(type_id: u32, types: &PortableRegistry) -> Result<String> {
    types
        .resolve(type_id)
        .map(|t| format_type(t, types))
        .unwrap_or_else(|| bail!("Invalid metadata"))
}

/// Formats generic type parameters into a Rust-like string representation.
///
/// This function helps `format_type()` handle generic types, ensuring that type parameters
/// are formatted correctly when they exist. If a type has generic parameters, they are enclosed
/// in angle brackets (e.g., `<T, U>`).
///
/// # How It Works With Other Functions:
/// - Called inside `format_type()` to process generic type parameters.
/// - Uses `resolve_type()` to retrieve and format each generic type.
///
fn format_generics(
    params: &[scale_info::TypeParameter<PortableForm>],
    types: &PortableRegistry,
) -> Result<String> {
    if params.is_empty() {
        Ok(String::default())
    } else {
        let generics = params
            .iter()
            .map(|p| {
                p.ty.map(|ty| resolve_type(ty.id, types))
                    .unwrap_or_else(|| Ok(String::default()))
            })
            .collect::<Result<Vec<String>>>()?;
        Ok(format!("<{}>", generics.join(", ")))
    }
}

fn parse_storage_entry(
    variant: &frame_metadata::v16::StorageEntryMetadata<scale_info::form::PortableForm>,
    types: &PortableRegistry,
) -> Result<Storage> {
    match &variant.ty {
        StorageEntryType::Map { key, value, .. } => {
            let type_key = resolve_type(key.id, types)?;
            let type_value = resolve_type(value.id, types)?;
            Ok(Storage {
                documentation: variant.docs.join("\n"),
                name: variant.name.clone(),
                type_key,
                type_value,
            })
        }
        StorageEntryType::Plain(v) => {
            let type_value = resolve_type(v.id, types)?;
            let type_value = if let StorageEntryModifier::Optional = &variant.modifier {
                format!("Option<{}>", type_value)
            } else {
                type_value
            };
            Ok(Storage {
                documentation: variant.docs.join("\n"),
                name: variant.name.clone(),
                type_key: String::default(),
                type_value,
            })
        }
    }
}

fn get_from_metadata_v16(
    metadata_v16: frame_metadata::v16::RuntimeMetadataV16,
) -> Result<RuntimePallets> {
    println!("Number of pallets: {}", metadata_v16.pallets.len());
    let mut pallets = Vec::new();
    for pallet in metadata_v16.pallets {
        let mut type_name: String = Default::default();
        let calls_type_def = if let Some(calls) = pallet.calls {
            let Some(calls_type) = metadata_v16.types.resolve(calls.ty.id) else {
                bail!("Invalid metadata")
            };
            type_name = calls_type
                .path
                .segments
                .first()
                .expect("cannot decode pallet type")
                .to_string();
            Some(calls_type.type_def.clone())
        } else {
            println!("{}: {} (0 calls)", pallet.index, pallet.name);
            None
        };
        let events_type_def = if let Some(events) = pallet.event {
            let Some(events_type) = metadata_v16.types.resolve(events.ty.id) else {
                bail!("Invalid metadata")
            };
            Some(events_type.type_def.clone())
        } else {
            println!("{}: {} (0 events)", pallet.index, pallet.name);
            None
        };
        let errors_type_def = if let Some(errors) = pallet.error {
            let Some(errors_type) = metadata_v16.types.resolve(errors.ty.id) else {
                bail!("Invalid metadata")
            };
            Some(errors_type.type_def.clone())
        } else {
            println!("{}: {} (0 errors)", pallet.index, pallet.name);
            None
        };

        let storages = pallet
            .storage
            .map(|storage| {
                storage
                    .entries
                    .iter()
                    .map(|v| parse_storage_entry(v, &metadata_v16.types))
                    .collect::<Result<Vec<Storage>>>()
            })
            .unwrap_or_else(|| {
                println!("{}: {} (0 storage)", pallet.index, pallet.name);
                Ok(Vec::default())
            })?;

        let constants = pallet
            .constants
            .iter()
            .map(|i| {
                let type_value = resolve_type(i.ty.id, &metadata_v16.types)?;
                let value = scale_value::scale::decode_as_type(
                    &mut &*i.value,
                    &i.ty.id,
                    &metadata_v16.types,
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
                Ok(Constant {
                    documentation: i.docs.join("\n"),
                    name: i.name.clone(),
                    value: value.to_string(),
                    type_value,
                })
            })
            .collect::<Result<Vec<Constant>>>()?;

        let pallet = Pallet::new(
            pallet.index,
            pallet.name.clone(),
            type_name,
            &calls_type_def,
            &events_type_def,
            &errors_type_def,
            storages,
            constants,
        )?;

        println!(
            "{}: {} ({} calls)",
            pallet.index,
            pallet.name,
            pallet.calls.len()
        );
        println!(
            "{}: {} ({} events)",
            pallet.index,
            pallet.name,
            pallet.events.len()
        );
        println!(
            "{}: {} ({} errors)",
            pallet.index,
            pallet.name,
            pallet.errors.len()
        );
        println!(
            "{}: {} ({} storages)",
            pallet.index,
            pallet.name,
            pallet.storages.len()
        );
        println!(
            "{}: {} ({} constants)",
            pallet.index,
            pallet.name,
            pallet.constants.len()
        );
        pallets.push(pallet);
    }
    Ok(pallets)
}

fn get_weights(max_weight: u128) -> Result<HashMap<String, HashMap<String, WeightInfo>>> {
    analyze_weight(
        Path::new(WEIGHT_FILEPATH),
        &MaxBlockWeight::new(max_weight as f64),
    )
    .map_err(|e| anyhow::anyhow!(e))
}

/// use template to render markdown file with runtime calls documentation
fn print_runtime(pallets: RuntimePallets) -> (String, String, String, String, String, String) {
    // init variables
    // -- user calls
    let mut user_calls_counter = 0;
    let user_calls_pallets: RuntimePallets = pallets
        .iter()
        .cloned()
        .filter_map(|mut pallet| {
            let pallet_name = pallet.name.clone();
            pallet
                .calls
                .retain(|call| CallCategory::is_user(&pallet_name, &call.name));
            if pallet.calls.is_empty() {
                None
            } else {
                user_calls_counter += pallet.calls.len();
                Some(pallet)
            }
        })
        .collect();

    // event counter
    let mut event_counter = 0;
    pallets
        .iter()
        .for_each(|pallet| event_counter += pallet.events.len());

    // error counter
    let mut error_counter = 0;
    pallets
        .iter()
        .for_each(|pallet| error_counter += pallet.errors.len());

    // storage counter
    let mut storage_counter = 0;
    pallets
        .iter()
        .for_each(|pallet| storage_counter += pallet.storages.len());

    // constant counter
    let mut constant_counter = 0;
    pallets
        .iter()
        .for_each(|pallet| constant_counter += pallet.constants.len());

    // compile template
    let tera = match Tera::new(TEMPLATES_GLOB) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    // fills tera context for rendering calls
    let mut context = tera::Context::new();
    context.insert("user_calls_counter", &user_calls_counter);
    context.insert("user_calls_pallets", &user_calls_pallets);

    let call_doc = tera
        .render("runtime-calls.md", &context)
        .expect("template error");

    // render events
    context.insert("pallets", &pallets);
    context.insert("event_counter", &event_counter);
    let event_doc = tera
        .render("runtime-events.md", &context)
        .expect("template error");

    // render errors
    context.insert("error_counter", &error_counter);
    let error_doc = tera
        .render("runtime-errors.md", &context)
        .expect("template error");

    let error_po = tera
        .render("runtime-errors.po", &context)
        .expect("template error");

    // render storages
    context.insert("pallets", &pallets);
    context.insert("storage_counter", &event_counter);
    let storage_doc = tera
        .render("runtime-storages.md", &context)
        .expect("template storage");

    // render constant
    context.insert("pallets", &pallets);
    context.insert("constant_counter", &constant_counter);
    let constant_doc = tera
        .render("runtime-constants.md", &context)
        .expect("template constant");

    (
        call_doc,
        event_doc,
        error_doc,
        error_po,
        storage_doc,
        constant_doc,
    )
}
