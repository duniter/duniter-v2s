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

fn main() {
    #[cfg(feature = "std")]
    {
        #[cfg(not(feature = "metadata-hash"))]
        substrate_wasm_builder::WasmBuilder::init_with_defaults().build();

        #[cfg(feature = "metadata-hash")]
        substrate_wasm_builder::WasmBuilder::init_with_defaults()
            .enable_metadata_hash("ĞD", 2)
            .build();
    }
}
