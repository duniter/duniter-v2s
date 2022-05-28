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

#![cfg_attr(not(feature = "std"), no_std)]

/// Bound length; forbid trailing or double spaces; accept only ascii alphanumeric or punctuation or space
pub fn validate_idty_name(idty_name: &[u8]) -> bool {
    idty_name.len() >= 3
        && idty_name.len() <= 64
        && idty_name[0] != 32
        && idty_name[idty_name.len() - 1] != 32
        && idty_name
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || *c == 32)
        && idty_name
            .iter()
            .zip(idty_name.iter().skip(1))
            .all(|(c1, c2)| *c1 != 32 || *c2 != 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_idty_name() {
        assert!(validate_idty_name(b"B0b"));
        assert!(validate_idty_name(b"lorem ipsum dolor-sit_amet."));
        assert!(!validate_idty_name(b" space"));
        assert!(!validate_idty_name(b"space "));
        assert!(!validate_idty_name(b"double  space"));
        assert!(!validate_idty_name("non-ascii🌵".as_bytes()));
        assert!(!validate_idty_name("ğune".as_bytes()));
    }
}
