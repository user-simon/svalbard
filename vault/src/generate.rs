//! Defines all generative algorithms used.

use std::iter;

use argon2;
use rand::Rng;

use crate::seed::*;

// IDEAS:
// password length range instead of fixed
//
// simplify seed variables with templates, like:
// PIN    => --N--, <length>
// BASIC  => -LN--, length 15-20
// MEDIUM => ULNS-, length 20-35

struct PasswordTable {
    target_len: usize,
    sets: Vec<&'static[u8]>,
    rows: Vec<Vec<(usize, u8)>>,
}

impl PasswordTable {
    fn new(target_len: u8, sets: Vec<&'static [u8]>, digest: &[u8]) -> PasswordTable {
        let target_len = target_len as usize;
        let char_count = sets.iter().map(|set| set.len()).sum::<usize>();
        let mut rows = vec![vec![]; sets.len()];

        
        
        for (i, chunk) in digest.chunks_exact(2).enumerate() {
            if let &[set_seed, char_seed] = chunk {
                let set_idx = {
                    let mut i = set_seed as usize % char_count;
                    let mut set_index = 0;
                    
                    for set in &sets {
                        if i >= set.len() {
                            i -= set.len();
                            set_index += 1;
                        } else {
                            break;
                        }
                    };
                    set_index
                };
                rows[set_idx].push((i, char_seed));
            } else {
                unreachable!()
            }
        };
        PasswordTable {
            target_len,
            sets,
            rows,
        }
    }

    fn balance(mut self) -> Self {
        let min_freq = 2.min(self.target_len as usize / self.sets.len());
        let compensations: Vec<usize> = self.rows.iter()
            .enumerate()
            .filter_map(|(i, row)| {
                if row.len() < min_freq {
                    Some((i, min_freq - row.len()))
                } else {
                    None
                }
            })
            .flat_map(|(i, needed)| iter::repeat(i).take(needed))
            .collect();
        
        for i in compensations {
            let cell = self.rows.iter_mut()
                .max_by(|a, b| a.len().cmp(&b.len()))
                .and_then(|row| row.pop())
                .unwrap();
            self.rows[i].push(cell);
        }
        for row in &self.rows {
            debug_assert!(row.len() >= min_freq);
        };
        self
    }

    fn build(self) -> String {
        todo!()
    }
}

/// Derives a password using the given parameters.
///
/// * `key` - Specific to the user, essentially equivalent to a master password.
/// * `pepper` - Specific to the [Vault](crate::Vault).
/// * `seed` - Specific to the password.
///
/// # Algorithm overview
///
/// 1. Concatenate the key and the seed identifier.
/// 2. Hash using [argon2d](argon2) with the following parameters:
///     * secret: `pepper`,
///     * salt: `seed.salt`,
///     * output length: `u32::max(seed.length, 4)`.
/// 3.
pub fn password(key: &str, pepper: &[u8], seed: &Seed) -> String {
    let target_len = seed.length as usize;
    let digest = {
        use argon2::*;

        let mut config = Config::default();
        config.hash_length = 4.max(target_len * 2) as u32;
        config.secret = pepper;
        config.variant = Variant::Argon2d;

        let data = format!("{}{}", key, seed.identifier);
        hash(&data, &seed.salt.to_be_bytes(), config)
    };

    PasswordTable::new(seed.length, seed.characters.get(), &digest)
        .balance()
        .build()
    
    // let mut seed_table = digest.chunks_exact(3).map(|chunk| {
    //     if let &[set_seed, char_seed, shuffle_seed] = chunk {
    //         let set_idx = get_set_idx(set_seed as usize % domain_char_count);
    //         let shuffle_idx = shuffle_seed as usize % target_len;
    //         (set_idx, char_seed, shuffle_idx)
    //     } else {
    //         unreachable!()
    //     }
    // });
    
    // let char_seeds = digest.chunks_exact(3).map(|chunk| {
    //     if let &[set_seed, char_seed, pos_seed] = chunk {
    //         let set_idx = get_set_idx(set_seed as usize);

    //         todo!()
    //         // (outer_seed, inner_seed, pos_seed)
    //     } else {
    //         unreachable!()
    //     }
    // });

    // let min_set_freq = 2.min(target_len / sets.len());


    // let (mut password, freq_table) = {
    //     let mut freq_table = vec![0_u8; sets.len()];
    //     let mut password = Vec::with_capacity(target_len);
    //     let char_count: usize = sets.iter()
    //         .map(|s| s.len())
    //         .sum();

    //     for i in 0..target_len {
    //         let index = digest[i] as usize % char_count;
    //         let (set_idx, char_idx) = split_index(index);
    //         freq_table[set_idx] += 1;
    //         password.push(sets[set_idx][char_idx]);
    //     }
    //     (password, freq_table)
    // };

    // let freq_comps = freq_table.iter()
    //     .enumerate()
    //     .filter_map(|(set, &freq)| {
    //         let needed = min_set_freq
    //             .checked_sub(freq as usize)
    //             .unwrap_or(0);

    //         if needed == 0 {
    //             None
    //         } else {
    //             Some((set, needed))
    //         }
    //     })
    //     .flat_map(|(set, needed)| iter::repeat(set).take(needed));

    // for (set_idx, digest_data) in freq_comps.zip(digest.rchunks_exact(2)) {
    //     if let &[char_seed, insert_seed] = digest_data {
    //         let set = sets[set_idx];
    //         let char_idx = char_seed as usize % set.len();
    //         let insert_idx = insert_seed as usize % target_len;

    //         password[insert]

    //         let char_idx = char_seed as usize % target_len;
    //         let insert_idx = insert_seed as usize % target_len;
    //         password[insert_idx] = sets[set][set_char_idx];
    //     } else {
    //         unreachable!()
    //     }
    // }

    // let sets = seed.characters.get();
    // let digest = {
    //     let needed_bytes = seed.length + sets.len() as u32 * 2; // extra bytes to ensure adequate set representation

    //     let mut config = argon2::Config::default();
    //     config.hash_length = needed_bytes.max(4); // argon2 requires at least 4 bytes output
    //     config.secret = pepper;
    //     config.variant = argon2::Variant::Argon2d;

    //     let data = format!("{}{}", key, seed.identifier);

    //     self::hash(&data, &seed.salt.to_be_bytes(), config)
    // };

    // let target_len = seed.length as usize;

    // // utility to split index to (set_index, offset)
    // let split_index = |i: usize| -> (usize, usize) {
    //     let mut offset = i;
    //     let mut set_index = 0;

    //     for set in &sets {
    //         if offset >= set.len() {
    //             offset -= set.len();
    //             set_index += 1;
    //         } else {
    //             break;
    //         }
    //     };
    //     (set_index, offset)
    // };

    // // build the seed table from the first half of the digest
    // let seed_table = {
    //     let mut table = vec![vec![]; sets.len()];
    //     let total_set_len: usize = sets.iter()
    //         .map(|s| s.len())
    //         .sum();

    //     // build initial table
    //     for i in 0..target_len {
    //         let seed = digest[i] as usize % total_set_len;
    //         let (set_index, offset) = split_index(seed);
    //         table[set_index].push((offset, seed));
    //     };

    //     // determine which sets are underrepresented and by how much
    //     let min_freq = 2.min(target_len / sets.len());
    //     let freq_comps: Vec<usize> = table.iter()
    //         .map(|set| {
    //             let freq = set.len();
    //             min_freq.max(freq) - freq
    //         })
    //         .enumerate()
    //         .filter(|&(_, need)| need > 0)
    //         .flat_map(|(set_index, need)| std::iter::repeat(set_index).take(need))
    //         .collect();

    //     // rebalance the table to ensure set adequate representation
    //     for set_index in freq_comps {
    //         let (offset, seed) = table.iter_mut()
    //             .max_by(|a, b| a.len().cmp(&b.len()))
    //             .unwrap()
    //             .pop()
    //             .unwrap();
    //         // table[set_index].push();
    //     };
    //     table
    // };

    // seed_table.iter();

    // todo!()

    // let min_freq = 2.min(target_len / sets.len());
    // let needed_freqs = char_table.iter()
    //     .map(|set| {
    //         let freq = set.len();
    //         min_freq.max(freq) - freq
    //     });

    // // make room
    // for _ in 0..needed_freqs.sum() {
    //     let over_rep_set = char_table.iter_mut()
    //         .filter(|s| s.len() > min_freq)
    //         .next()
    //         .unwrap();
    //     over_rep_set.pop();
    // }

    // let target_length = seed.length as usize;
    // let sets = seed.characters.get();
    // let chars_count: usize = sets.iter()
    //     .map(|s| s.len())
    //     .sum();

    // // utility to split index to (set_index, offset)
    // let split_index = |i: usize| -> (usize, usize) {
    //     let mut offset = i;
    //     let mut set_index = 0;

    //     for set in &sets {
    //         if offset >= set.len() {
    //             offset -= set.len();
    //             set_index += 1;
    //         } else {
    //             break;
    //         }
    //     }
    //     (set_index, offset)
    // };

    // // create all characters and keep track of absolute set frequency
    // let (mut password, frequencies) = {
    //     let mut frequencies = vec![0_u32; sets.len()];
    //     let password = (0..target_length).map(|i| {
    //         let char_index = digest[i] as usize % chars_count;
    //         let (set_index, offset) = split_index(char_index);
    //         frequencies[set_index] += 1;
    //         sets[set_index][offset]
    //     }).collect();

    //     (password, frequencies)
    // };

    // // calculate the needed frequencies for each set
    // let needed_frequencies = {
    //     let wanted = 2.min(target_length / sets.len()) as u32;
    //     frequencies.iter()
    //         .map(move |&f| wanted.max(f) - f)
    //         .enumerate()
    //         .filter(|&(_, f)| f > 0)
    //         .flat_map(|(s, f)|
    //             (0..f).map(move |i| (i, s))
    //         )
    // };

    // // fulfill the needs
    // for (i, set_index) in needed_frequencies {

    // }

    // String::from_utf8(password).unwrap()
}

/// Generates a new pepper value.
pub fn pepper() -> Vec<u8> {
    const LENGTH: usize = 20;

    let mut rng = rand::thread_rng();
    let mut buffer = vec![0_u8; LENGTH];
    rng.fill(buffer.as_mut_slice());
    buffer
}

/// Generates an authentication token from a key.
///
/// Internally, hashes the key using [argon2].
pub fn auth_token(key: &str, vault_pepper: &[u8]) -> Vec<u8> {
    self::hash(key, vault_pepper, argon2::Config::default())
}

/// Utility function to hash data using [argon2].
fn hash(data: &str, salt: &[u8], config: argon2::Config) -> Vec<u8> {
    argon2::hash_raw(&data.as_bytes(), &salt, &config).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password() {
        let mut seed = Seed {
            identifier: "".to_string(),
            length: 255,
            salt: 2,
            characters: Characters::all(),
            username: None,
        };
        super::password("", b"", &seed);

        // for i in 0..100 {
        //     seed.salt = i;
        //     dbg!(super::password("", b"", &seed));
        // }
    }
}
