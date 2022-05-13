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

use anyhow::{bail, Context, Result};
use codec::Decode;
use scale_info::form::PortableForm;
use std::{
    fs::File,
    io::{Read, Write},
};

const CALLS_DOC_FILEPATH: &str = "docs/api/runtime-calls.md";

type RuntimeCalls = Vec<Pallet>;

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
            ("Membership", "force_request_membership") => Self::Root,
            ("Membership", "claim_membership" | "revoke_membership") => Self::Disabled,
            ("Cert", "force_add_cert" | "del_cert" | "remove_all_certs_received_by") => Self::Root,
            ("SmithsMembership", "force_request_membership") => Self::Root,
            ("SmithsMembership", "claim_membership") => Self::Disabled,
            ("SmithsCert", "force_add_cert" | "del_cert" | "remove_all_certs_received_by") => {
                Self::Root
            }
            ("SmithsCollective", "set_members" | "disapprove_proposal") => Self::Root,
            ("Utility", "dispatch_as") => Self::Root,
            _ => Self::User,
        }
    }
    fn is_root(pallet_name: &str, call_name: &str) -> bool {
        matches!(Self::is(pallet_name, call_name), Self::Root)
    }
    fn is_user(pallet_name: &str, call_name: &str) -> bool {
        matches!(Self::is(pallet_name, call_name), Self::User)
    }
}

#[derive(Clone)]
struct Pallet {
    index: u8,
    name: String,
    calls: Vec<Call>,
}

impl Pallet {
    fn new(
        index: u8,
        name: String,
        scale_type_def: &scale_info::TypeDef<PortableForm>,
    ) -> Result<Self> {
        if let scale_info::TypeDef::Variant(calls_enum) = scale_type_def {
            Ok(Self {
                index,
                name,
                calls: calls_enum.variants().iter().map(Into::into).collect(),
            })
        } else {
            bail!("Invalid metadata")
        }
    }
}

#[derive(Clone)]
struct Call {
    docs: Vec<String>,
    index: u8,
    name: String,
    params: Vec<CallParam>,
}

impl From<&scale_info::Variant<PortableForm>> for Call {
    fn from(variant: &scale_info::Variant<PortableForm>) -> Self {
        Self {
            docs: variant.docs().to_vec(),
            index: variant.index(),
            name: variant.name().to_owned(),
            params: variant.fields().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone)]
struct CallParam {
    name: String,
    type_name: String,
}

impl From<&scale_info::Field<PortableForm>> for CallParam {
    fn from(field: &scale_info::Field<PortableForm>) -> Self {
        Self {
            name: field.name().cloned().unwrap_or_default(),
            type_name: field.type_name().cloned().unwrap_or_default(),
        }
    }
}

pub(super) fn gen_calls_doc() -> Result<()> {
    // Read metadata
    let mut file = std::fs::File::open("resources/metadata.scale")
        .with_context(|| "Failed to open metadata file")?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .with_context(|| "Failed to read metadata file")?;

    let metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut &bytes[..])
        .with_context(|| "Failed to decode metadata")?;

    println!("Metadata successfully loaded!");

    let runtime_calls = if let frame_metadata::RuntimeMetadata::V14(metadata_v14) = metadata.1 {
        get_calls_from_metadata_v14(metadata_v14)?
    } else {
        bail!("unsuported metadata version")
    };

    let output = print_runtime_calls(runtime_calls);

    let mut file = File::create(CALLS_DOC_FILEPATH)
        .with_context(|| format!("Failed to create file '{}'", CALLS_DOC_FILEPATH))?;
    file.write_all(output.as_bytes())
        .with_context(|| format!("Failed to write to file '{}'", CALLS_DOC_FILEPATH))?;

    Ok(())
}

fn get_calls_from_metadata_v14(
    metadata_v14: frame_metadata::v14::RuntimeMetadataV14,
) -> Result<RuntimeCalls> {
    println!("Number of pallets: {}", metadata_v14.pallets.len());
    let mut pallets = Vec::new();
    for pallet in metadata_v14.pallets {
        if let Some(calls) = pallet.calls {
            if let Some(calls_type) = metadata_v14.types.resolve(calls.ty.id()) {
                let pallet = Pallet::new(pallet.index, pallet.name.clone(), calls_type.type_def())?;
                let calls_len = pallet.calls.len();
                println!("{}: {} ({} calls)", pallet.index, pallet.name, calls_len);
                pallets.push(pallet);
            } else {
                bail!("Invalid metadata")
            }
        } else {
            println!("{}: {} (0 calls)", pallet.index, pallet.name);
        }
    }
    Ok(pallets)
}

fn print_runtime_calls(pallets: RuntimeCalls) -> String {
    let mut user_calls_counter = 0;
    let user_calls_pallets: RuntimeCalls = pallets
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
    let root_calls_pallets: RuntimeCalls = pallets
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

    let mut output = String::new();

    output.push_str("# Runtime calls\n\n");
    output.push_str("Calls are categorized according to the dispatch origin they require:\n\n");
    output.push_str(
        r#"1. User calls: the dispatch origin for this kind of call must be Signed by
the transactor. This is the only call category that can be submitted with an extrinsic.
"#,
    );
    output.push_str(
        r#"1. Root calls: This kind of call requires a special origin that can only be invoked
through on-chain governance mechanisms.
"#,
    );
    output.push_str(
        r#"1. Inherent calls: This kind of call is invoked by the author of the block itself
(usually automatically by the node).
"#,
    );

    output.push_str("\n\n## User calls\n\n");
    output.push_str(&print_calls_category(
        user_calls_counter,
        "user",
        user_calls_pallets,
    ));

    output.push_str("\n\n## Root calls\n\n");
    output.push_str(&print_calls_category(
        root_calls_counter,
        "root",
        root_calls_pallets,
    ));

    output
}

fn print_calls_category(calls_counter: usize, category_name: &str, pallets: Vec<Pallet>) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "There are **{}** {} calls organized in **{}** pallets.\n",
        calls_counter,
        category_name,
        pallets.len()
    ));

    for pallet in pallets {
        output.push_str(&format!("\n### {}: {}\n\n", pallet.index, pallet.name));
        for call in pallet.calls {
            output.push_str(&format!(
                "<details><summary>{}: {}({})</summary>\n<p>\n\n{}</p>\n</details>\n\n",
                call.index,
                call.name,
                print_call_params(&call.params),
                print_call_details(&call),
            ));
        }
    }
    output
}

fn print_call_details(call: &Call) -> String {
    let mut output = String::new();
    output.push_str(&format!("### Index\n\n`{}`\n\n", call.index));
    output.push_str(&format!(
        "### Documentation\n\n{}\n\n",
        call.docs
            .iter()
            .take_while(|line| !line.starts_with("# <weight>"))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    ));
    if !call.params.is_empty() {
        output.push_str("### Types of parameters\n\n```rust\n");
        output.push_str(
            &call
                .params
                .iter()
                .map(|param| format!("{}: {}", param.name, param.type_name))
                .collect::<Vec<_>>()
                .join(",\n"),
        );
        output.push_str("\n```\n\n");
    }
    output
}

fn print_call_params(call_params: &[CallParam]) -> String {
    call_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}
