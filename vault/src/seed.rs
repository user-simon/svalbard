//! Contains data used to seed the [generate::password](crate::generate::password) algorithm.

use serde::{Serialize, Deserialize};
use bitflags::bitflags;

bitflags! {
    /// Utility to specify what character sets should be used in a [Seed].
    #[derive(Serialize, Deserialize)]
    pub struct Characters: u8 {
        const UPPER_CASE = 1 << 0;
        const LOWER_CASE = 1 << 1;
        const NUMERICAL  = 1 << 2;
        const SPECIAL    = 1 << 3;
        const RARE       = 1 << 4;
    }
}

impl Characters {
    /// Defines the available character sets to be used when encoding a password.
    /// 
    /// Note that the following characters have been filtered out as they may be confused for one
    /// another: `I, O, l, 0`.
    pub const SETS: [&'static [u8]; 5] = [
        b"ABCDEFGHJKLMNPQRSTUVWXYZ",
        b"abcdefghijkmnopqrstuvwxyz",
        b"123456789",
        b"!#&()*+,-.<=>?@[]_",
        b"\"$%/:;\\^{|}~ ",
    ];
    
    /// Gets the [String] forms of all character sets held.
    pub fn get(&self) -> Vec<&[u8]> {
        Characters::SETS.iter()
            .enumerate()
            .filter(|(i, _)| self.bits & (1 << i) != 0)
            .map(|(_, &string)| string)
            .collect()
    }
}

impl ToString for Characters {
    fn to_string(&self) -> String {
        const FLAG_CHARS: &str = "ULNSR";
        FLAG_CHARS.char_indices()
            .map(|(i, c)| {
                if self.bits & (1 << i) != 0 {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    }
}

/// Contains all parameters used to generate passwords.
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Seed {
    /// Unique seed identifier, e.g. "GitHub".
    pub identifier: String,
    /// Specifies length.
    pub length: u32,
    /// Facilitates modifying output without changing other parameters. Does not have to be
    /// cryptographically secure.
    pub salt: u64,
    /// Specifies character sets to be used.
    pub characters: Characters,
    /// Contains username for service. Provided for convenience only; does not participate in
    /// output.
    pub username: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: &[u8] = Characters::SETS[0];
    const L: &[u8] = Characters::SETS[1];
    const N: &[u8] = Characters::SETS[2];
    const S: &[u8] = Characters::SETS[3];
    const R: &[u8] = Characters::SETS[4];

    #[test]
    fn character_get() {
        assert_eq!(Characters::UPPER_CASE.get(), [U]);
        assert_eq!(Characters::LOWER_CASE.get(), [L]);
        assert_eq!(Characters::NUMERICAL.get(),  [N]);
        assert_eq!(Characters::SPECIAL.get(),    [S]);
        assert_eq!(Characters::RARE.get(),       [R]);

        assert_eq!(
            (Characters::UPPER_CASE | Characters::LOWER_CASE).get(),
            [U, L]
        );
        assert_eq!(
            (Characters::LOWER_CASE | Characters::NUMERICAL).get(),
            [L, N]
        );
        assert_eq!(
            (Characters::LOWER_CASE | Characters::NUMERICAL | Characters::SPECIAL).get(),
            [L, N, S]
        );
        assert_eq!(
            Characters::all().get(),
            [U, L, N, S, R]
        );
    }

    #[test]
    fn characters_to_string() {
        let data = [
            (Characters::LOWER_CASE, "-L---"),
            (Characters::UPPER_CASE, "U----"),
            (Characters::UPPER_CASE | Characters::RARE, "U---R"),
            (Characters::all(), "ULNSR")
        ];

        for (set, str) in data {
            assert_eq!(set.to_string(), str);
        }
    }
}
