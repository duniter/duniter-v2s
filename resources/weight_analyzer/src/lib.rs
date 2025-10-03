use convert_case::{Case, Casing};
use glob::glob;
use serde::Serialize;
use std::{collections::HashMap, ops::Div, path::Path};
use subweight_core::{
    parse::{
        overhead::Weight,
        pallet::{ChromaticExtrinsic, ComponentRange},
        storage::Weights,
    },
    scope::Scope,
    term::Term,
};

// Substrate default maximum weight of a block in nanoseconds.
// Since the maximum block weight is one-third of the execution time,
// it corresponds to a block time of 6 seconds.
const MAX_BLOCK_WEIGHT_NS: f64 = 2_000_000_000_000.;

pub struct MaxBlockWeight(f64);
impl Default for MaxBlockWeight {
    fn default() -> Self {
        MaxBlockWeight(MAX_BLOCK_WEIGHT_NS)
    }
}
impl Div<&MaxBlockWeight> for f64 {
    type Output = Self;

    fn div(self, max_block_weight: &MaxBlockWeight) -> Self::Output {
        self / max_block_weight.0
    }
}
impl MaxBlockWeight {
    pub fn new<T: Into<f64>>(value: T) -> Self {
        MaxBlockWeight(value.into())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct WeightInfo {
    pub weight: u128,
    pub relative_weight: f64,
}

/// Returns a HashMap <pallet_name, <extrinsic_name, weigh_info>>
/// of the analyzed weights.
///
/// # Arguments
///
///   * `folder_path` - A Path to a folder where the weight files are stored.
///     `paritydb_weights.rs` is mandatory and pallet weights should start by
///     `pallet_`.
///   * `max_block_weight` - The maximal weight of a block.
///
/// # Examples
///
/// ```
///    use weightanalyzer::analyze_weight;
///    use std::path::Path;
///    use weightanalyzer::MaxBlockWeight;
///    let weight_by_pallet = analyze_weight(Path::new("../../runtime/gdev/src/weights/"), &MaxBlockWeight::default());
///    println!("{:?}", weight_by_pallet);
/// ```
pub fn analyze_weight(
    folder_path: &Path,
    max_block_weight: &MaxBlockWeight,
) -> Result<HashMap<String, HashMap<String, WeightInfo>>, String> {
    let pallet_weights = read_pallet_weight(folder_path)?;
    let db_weight = read_db_weight(folder_path)?;
    let overhead_weights = read_overhead_weight(folder_path)?;

    // Initialize scope with db weights
    let mut scope = Scope::from_substrate();
    scope = scope.with_storage_weights(db_weight.weights.read, db_weight.weights.write);

    process(pallet_weights, scope, max_block_weight, &overhead_weights)
}

fn read_pallet_weight(folder_path: &Path) -> Result<Vec<Vec<ChromaticExtrinsic>>, String> {
    let mut parsed_files = Vec::new();
    for path in glob(folder_path.join("*").to_str().expect("Invalid pallet path"))
        .expect("Invalid pallet pattern")
        .filter_map(Result::ok)
    {
        let file = subweight_core::parse::pallet::parse_file(&path);
        if let Ok(file) = file {
            parsed_files.push(file);
        }
    }
    if parsed_files.is_empty() {
        return Err("No pallet found".into());
    }
    Ok(parsed_files)
}

fn read_db_weight(folder_path: &Path) -> Result<Weights, String> {
    subweight_core::parse::storage::parse_file(folder_path.join("paritydb_weights.rs").as_path())
}

fn read_overhead_weight(folder_path: &Path) -> Result<Weight, String> {
    subweight_core::parse::overhead::parse_file(folder_path.join("extrinsic_weights.rs").as_path())
}

fn evaluate_weight(
    extrinsic: ChromaticExtrinsic,
    scope: &mut Scope<Term<u128>>,
    max_block_weight: &MaxBlockWeight,
    overhead: &Weight,
) -> Result<(String, String, WeightInfo), String> {
    // Extend the scope with the maximum value of the complexity parameter.
    if let Some(params) = extrinsic.comp_ranges {
        params
            .iter()
            .for_each(|(key, val): (&String, &ComponentRange)| {
                scope.put_var(key.as_str(), Term::Scalar(val.max.into()));
            });
    }

    // Evaluate the weight
    let mut weight = extrinsic
        .term
        .simplify(subweight_core::Dimension::Time)
        .expect("Can't evaluate")
        .eval(scope)?;

    // Add base extrinsic overhead
    if let Weight::ExtrinsicBase(i) = overhead {
        weight += i
            .simplify(subweight_core::Dimension::Time)
            .expect("Can't evaluate")
            .eval(scope)?;
    }

    let relative_weight = (weight as f64) / max_block_weight * 100.;
    Ok((
        extrinsic
            .pallet
            .to_case(Case::Title)
            .replace("Pallet", "")
            .replace(".rs", "")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect(),
        extrinsic.name,
        WeightInfo {
            weight,
            relative_weight,
        },
    ))
}

fn process(
    pallet_weights: Vec<Vec<ChromaticExtrinsic>>,
    mut scope: Scope<Term<u128>>,
    max_block_weight: &MaxBlockWeight,
    overhead: &Weight,
) -> Result<HashMap<String, HashMap<String, WeightInfo>>, String> {
    let mut weight_by_pallet: HashMap<String, HashMap<String, WeightInfo>> = HashMap::new();
    for i in pallet_weights {
        for j in i {
            let (pallet, extrinsic, weight) =
                evaluate_weight(j, &mut scope, max_block_weight, overhead)?;
            if let Some(i) = weight_by_pallet.get_mut(&pallet) {
                i.insert(extrinsic, weight);
            } else {
                weight_by_pallet.insert(pallet, HashMap::from([(extrinsic, weight)]));
            }
        }
    }
    Ok(weight_by_pallet)
}

#[cfg(test)]
mod tests {
    use crate::{MaxBlockWeight, analyze_weight};
    use std::path::Path;
    #[test]
    fn should_works() {
        let weight_by_pallet = analyze_weight(
            Path::new("../../runtime/gdev/src/weights/"),
            &MaxBlockWeight::default(),
        );
        assert!(
            weight_by_pallet
                .clone()
                .unwrap()
                .get("Balances")
                .unwrap()
                .len()
                == 10
        ); // 8 extrinsics in pallet
        println!("{:?}", weight_by_pallet); // cargo test  -- --nocapture
    }
    #[test]
    #[should_panic]
    fn should_not_works() {
        let _ = analyze_weight(Path::new(""), &MaxBlockWeight::default()).unwrap();
    }
}
