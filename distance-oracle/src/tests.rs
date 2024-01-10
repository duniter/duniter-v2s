// Copyright 2023 Axiom-Team
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

use dubp_wot::{
    data::rusty::RustyWebOfTrust, operations::distance::DistanceCalculator, WebOfTrust,
};
use flate2::read::ZlibDecoder;
use sp_runtime::Perbill;
use std::{fs::File, io::Read};

#[tokio::test]
#[ignore = "long to execute"]
async fn test_distance_against_v1() {
    let wot = wot_from_v1_file();
    let n = wot.size();
    let min_certs_for_referee = (wot.get_enabled().len() as f32).powf(1. / 5.).ceil() as u32;

    // Reference implementation
    let ref_calculator = dubp_wot::operations::distance::RustyDistanceCalculator;
    let t_a = std::time::Instant::now();
    let ref_results: Vec<Perbill> = wot
        .get_enabled()
        .into_iter()
        .chain(wot.get_disabled().into_iter())
        .zip(0..n)
        .map(|(i, _)| {
            let result = ref_calculator
                .compute_distance(
                    &wot,
                    dubp_wot::operations::distance::WotDistanceParameters {
                        node: i,
                        sentry_requirement: min_certs_for_referee,
                        step_max: 5,
                        x_percent: 0.8,
                    },
                )
                .unwrap();
            Perbill::from_rational(result.success, result.sentries)
        })
        .collect();
    println!("ref time: {}", t_a.elapsed().as_millis());

    // Our implementation
    let mut client = crate::api::client_from_wot(wot);
    client.pool_len = n;

    let t_a = std::time::Instant::now();
    let results = crate::run(&client, &Default::default(), false)
        .await
        .unwrap();
    println!("new time: {}", t_a.elapsed().as_millis());
    assert_eq!(results.0.len(), n);

    let mut errors: Vec<_> = results
        .0
        .iter()
        .zip(ref_results.iter())
        .map(|(r, r_ref)| r.deconstruct() as i64 - r_ref.deconstruct() as i64)
        .collect();
    errors.sort_unstable();
    println!(
        "Error: {:?} / {:?} / {:?} / {:?} / {:?}  (min / 1Q / med / 3Q / max)",
        errors[0],
        errors[errors.len() / 4],
        errors[errors.len() / 2],
        errors[errors.len() * 3 / 4],
        errors[errors.len() - 1]
    );

    let correct_results = results
        .0
        .iter()
        .zip(ref_results.iter())
        .map(|(r, r_ref)| (r == r_ref) as usize)
        .sum::<usize>();
    println!("Correct results: {correct_results} / {n}");
    assert_eq!(correct_results, n);
}

fn wot_from_v1_file() -> RustyWebOfTrust {
    let file = File::open("wot.deflate").expect("Cannot open wot.deflate");
    let mut decompressor = ZlibDecoder::new(file);
    let mut decompressed_bytes = Vec::new();
    decompressor
        .read_to_end(&mut decompressed_bytes)
        .expect("Cannot decompress wot.deflate");
    bincode::deserialize::<RustyWebOfTrust>(&decompressed_bytes).expect("Cannot decode wot.deflate")
}
