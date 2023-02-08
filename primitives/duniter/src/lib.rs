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

#![cfg_attr(not(feature = "std"), no_std)]

/// Rules for valid identity names are defined below
/// - Bound length to 42
/// - accept only ascii alphanumeric or - or _
pub fn validate_idty_name(idty_name: &[u8]) -> bool {
    idty_name.len() >= 3
        && idty_name.len() <= 42 // length smaller than 42
        // all characters are alphanumeric or - or _
        && idty_name
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'-' || *c == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_idty_name() {
        // --- allow
        assert!(validate_idty_name(b"B0b"));
        assert!(validate_idty_name(b"lorem_ipsum-dolor-sit_amet"));
        assert!(validate_idty_name(
            b"1_______10________20________30________40_-"
        ));
        // --- disallow
        assert!(!validate_idty_name(
            b"1_______10________20________30________40_-_"
        ));
        assert!(!validate_idty_name(b"with space"));
        assert!(!validate_idty_name("non-ascii🌵".as_bytes()));
        assert!(!validate_idty_name("ğune".as_bytes()));
        assert!(!validate_idty_name("toto!".as_bytes()));
    }
}
