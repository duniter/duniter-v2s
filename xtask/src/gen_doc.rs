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

use anyhow::{bail, Context, Result};
use codec::Decode;
use core::hash::Hash;
use scale_info::form::PortableForm;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::{
    fs::File,
    io::{Read, Write},
};
use tera::Tera;
use weightanalyzer::analyze_weight;
use weightanalyzer::MaxBlockWeight;
use weightanalyzer::WeightInfo;

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
const ERRORS_DOC_FILEPATH: &str = "docs/api/runtime-errors.md";
const TEMPLATES_GLOB: &str = "xtask/res/templates/*.md";
const WEIGHT_FILEPATH: &str = "runtime/common/src/weights/";

// define structs and implementations

type RuntimePallets = Vec<Pallet>;

#[derive(Clone, Serialize)]
struct Pallet {
    index: u8,
    name: String,
    calls: Vec<Call>,
    events: Vec<Event>,
    errors: Vec<ErroR>,
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

impl Pallet {
    fn new(
        index: u8,
        name: String,
        call_scale_type_def: &Option<scale_info::TypeDef<PortableForm>>,
        event_scale_type_def: &Option<scale_info::TypeDef<PortableForm>>,
        error_scale_type_def: &Option<scale_info::TypeDef<PortableForm>>,
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
            calls,
            events,
            errors,
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
            name: field.clone().name.unwrap_or_default(),
            type_name: field.clone().type_name.unwrap_or_default(),
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
            name: field.clone().name.unwrap_or_default(),
            type_name: field.clone().type_name.unwrap_or_default(),
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

enum CallCategory {
    Disabled,
    Inherent,
    OtherOrigin,
    Root,
    Sudo,
    User,
}

impl CallCategory {
    fn is(pallet_name: &str, call_name: &str) -> Self {
        match (pallet_name, call_name) {
            ("System", "remark" | "remark_with_event") => Self::Disabled,
            ("System", _) => Self::Root,
            ("Babe", "report_equivocation_unsigned") => Self::Inherent,
            ("Babe", "plan_config_change") => Self::Root,
            ("Timestamp", _) => Self::Inherent,
            ("Balances", "set_balance" | "force_transfer" | "force_unreserve") => Self::Root,
            ("AuthorityMembers", "prune_account_id_of" | "remove_member") => Self::Root,
            ("Authorship", _) => Self::Inherent,
            ("Session", _) => Self::Disabled,
            ("Grandpa", "report_equivocation_unsigned") => Self::Inherent,
            ("Grandpa", "note_stalled") => Self::Root,
            ("UpgradeOrigin", "dispatch_as_root") => Self::OtherOrigin,
            ("ImOnline", _) => Self::Inherent,
            ("Sudo", _) => Self::Sudo,
            (
                "Identity",
                "remove_identity" | "prune_item_identities_names" | "prune_item_identity_index_of",
            ) => Self::Root,
            ("Cert", "del_cert" | "remove_all_certs_received_by") => Self::Root,
            ("SmithCert", "del_cert" | "remove_all_certs_received_by") => Self::Root,
            ("TechnicalCommittee", "set_members" | "disapprove_proposal") => Self::Root,
            ("Utility", "dispatch_as") => Self::Root,
            ("Treasury", "approve_proposal" | "reject_proposal") => Self::OtherOrigin,
            _ => Self::User,
        }
    }

    fn is_root(pallet_name: &str, call_name: &str) -> bool {
        matches!(Self::is(pallet_name, call_name), Self::Root)
    }

    fn is_user(pallet_name: &str, call_name: &str) -> bool {
        matches!(Self::is(pallet_name, call_name), Self::User)
    }

    fn is_disabled(pallet_name: &str, call_name: &str) -> bool {
        matches!(Self::is(pallet_name, call_name), Self::Disabled)
    }
}

/// generate runtime calls documentation
pub(super) fn gen_doc() -> Result<()> {
    // Read metadata
    let mut file = std::fs::File::open("resources/metadata.scale")
        .with_context(|| "Failed to open metadata file")?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .with_context(|| "Failed to read metadata file")?;

    let metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &bytes[..])
        .with_context(|| "Failed to decode metadata")?;

    println!("Metadata successfully loaded!");

    let (mut runtime, max_weight) =
        if let frame_metadata::RuntimeMetadata::V15(ref metadata_v15) = metadata.1 {
            (
                get_from_metadata_v15(metadata_v15.clone())?,
                get_max_weight_from_metadata_v15(metadata_v15.clone())?,
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
                .expect(&("No weight for ".to_owned() + &pallet.name))
                .get(&call.name)
                .map_or(-1f64, |weight| {
                    (weight.relative_weight * 10000.).round() / 10000.
                })
        })
    });

    let (call_doc, event_doc, error_doc) = print_runtime(runtime);

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

    Ok(())
}

fn get_max_weight_from_metadata_v15(
    metadata_v15: frame_metadata::v15::RuntimeMetadataV15,
) -> Result<u128> {
    // Extract the maximal weight available in one block
    // from the metadata.
    let block_weights = metadata_v15
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
        &metadata_v15.types,
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

fn get_from_metadata_v15(
    metadata_v15: frame_metadata::v15::RuntimeMetadataV15,
) -> Result<RuntimePallets> {
    println!("Number of pallets: {}", metadata_v15.pallets.len());
    let mut pallets = Vec::new();
    for pallet in metadata_v15.pallets {
        let calls_type_def = if let Some(calls) = pallet.calls {
            let Some(calls_type) = metadata_v15.types.resolve(calls.ty.id) else {
                bail!("Invalid metadata")
            };
            Some(calls_type.type_def.clone())
        } else {
            println!("{}: {} (0 calls)", pallet.index, pallet.name);
            None
        };
        let events_type_def = if let Some(events) = pallet.event {
            let Some(events_type) = metadata_v15.types.resolve(events.ty.id) else {
                bail!("Invalid metadata")
            };
            Some(events_type.type_def.clone())
        } else {
            println!("{}: {} (0 events)", pallet.index, pallet.name);
            None
        };
        let errors_type_def = if let Some(errors) = pallet.error {
            let Some(errors_type) = metadata_v15.types.resolve(errors.ty.id) else {
                bail!("Invalid metadata")
            };
            Some(errors_type.type_def.clone())
        } else {
            println!("{}: {} (0 errors)", pallet.index, pallet.name);
            None
        };

        let pallet = Pallet::new(
            pallet.index,
            pallet.name.clone(),
            &calls_type_def,
            &events_type_def,
            &errors_type_def,
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
fn print_runtime(pallets: RuntimePallets) -> (String, String, String) {
    // init variables
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
    let mut root_calls_counter = 0;
    let root_calls_pallets: RuntimePallets = pallets
        .iter()
        .cloned()
        .filter_map(|mut pallet| {
            let pallet_name = pallet.name.clone();
            pallet
                .calls
                .retain(|call| CallCategory::is_root(&pallet_name, &call.name));
            if pallet.calls.is_empty() {
                None
            } else {
                root_calls_counter += pallet.calls.len();
                Some(pallet)
            }
        })
        .collect();
    let mut disabled_calls_counter = 0;
    let disabled_calls_pallets: RuntimePallets = pallets
        .iter()
        .cloned()
        .filter_map(|mut pallet| {
            let pallet_name = pallet.name.clone();
            pallet
                .calls
                .retain(|call| CallCategory::is_disabled(&pallet_name, &call.name));
            if pallet.calls.is_empty() {
                None
            } else {
                disabled_calls_counter += pallet.calls.len();
                Some(pallet)
            }
        })
        .collect();

    let mut event_counter = 0;
    pallets
        .iter()
        .for_each(|pallet| event_counter += pallet.events.len());

    let mut error_counter = 0;
    pallets
        .iter()
        .for_each(|pallet| error_counter += pallet.errors.len());

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
    context.insert("root_calls_counter", &root_calls_counter);
    context.insert("root_calls_pallets", &root_calls_pallets);
    context.insert("disabled_calls_counter", &disabled_calls_counter);
    context.insert("disabled_calls_pallets", &disabled_calls_pallets);

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

    (call_doc, event_doc, error_doc)
}
