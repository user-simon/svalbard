//! Defines the Svalbard back-end API.
//! 
//! Each password is generated based on three parameters:
//! 
//! * a key chosen by and specific to the user (a secret string specified by the user, essentially
//!   equivalent to a master password),
//! * a pepper specific to the [Vault] (a secret, locally stored pseudo-random byte sequence used to
//!   complement the user key),
//! * a [Seed] specific to the password (describes how the password should be generated).
//! 
//! This system of providing layer-specific data helps ensure the security and uniqueness of each
//! generated password. For more details, see the [password derivation](generate::password)
//! algorithm.

pub mod generate;
pub mod result;
pub mod seed;

use std::{
    path::{PathBuf, Path},
    fs,
};
use deunicode::AsciiChars;
use serde::{
    Serialize,
    Deserialize,
};
use serde_with::{
    serde_as,
    base64::Base64,
};
use self::{
    result::*,
    seed::*
};

/// Manages seeds and performs password generation.
/// 
/// Each vault is stored on file at the relative file path `vaults/{identifier}.vault`.
#[serde_as]
#[derive(Serialize, Deserialize, Hash)]
pub struct Vault {
    /// Contains path to vault on disk.
    #[serde(skip)]
    path: PathBuf,
    /// Unique vault identifier.
    identifier: String,
    /// Contains a pepper included when generating passwords.
    #[serde_as(as = "Base64")]
    pepper: Vec<u8>,
    /// Contains all seeds.
    seeds: Vec<Seed>,
    /// Authentication token generated from the user key
    #[serde_as(as = "Base64")]
    auth_token: Vec<u8>,
}

impl Vault {
    /// Creates a new [Vault] from an identifier.
    /// 
    /// # Errors
    /// * [`Error::VaultNameConflict`] if a [Vault] with given identifier already exists on disk.
    /// * [`Error::IO`] if creation of vault folder fails.
    pub fn new(vault_folder: &Path, identifier: String, key: &str) -> Result<Self> {
        fs::create_dir_all(vault_folder)
            .map_err(|e| Error::IO(e, vault_folder.to_owned()))?;

        let path = Vault::path_of(vault_folder, &identifier);
        let pepper = generate::pepper();
        let auth_token = generate::auth_token(key, &pepper);

        if path.exists() {
            Err(Error::VaultNameConflict(identifier))
        } else {
            let vault = Vault {
                path,
                identifier,
                seeds: Vec::new(),
                pepper,
                auth_token,
            };
            vault.save().map(|_| vault)
        }
    }
    
    /// Loads an existing [Vault] with given identifier from disk.
    /// 
    /// # Errors
    /// * [`Error::IO`] if [Vault] with given identifier does not exist.
    /// * [`Error::JSON`] if file contains corrupted data.
    pub fn load(vault_folder: &Path, identifier: String) -> Result<Self> {
        let path = Vault::path_of(vault_folder, &identifier);

        fs::read_to_string(&path)
            .map_err(|e| Error::IO(e, path.to_owned()))
            .and_then(|string|  {
                serde_json::from_str::<Vault>(&string)
                    .map_err(|e| Error::JSON(e, path.to_owned()))
            })
            .map(|mut vault| {
                vault.path = path;
                vault
            })
    }
    
    /// Saves [Vault] contents to disk.
    /// 
    /// # Errors
    /// * [`Error::JSON`] on internal [`serde_json`] errors.
    /// * [`Error::IO`] if file could not be written to.
    pub fn save(&self) -> Result<()> {
        let string = serde_json::to_string_pretty(self)
            .unwrap();
        fs::write(&self.path, string)
            .map_err(|e| Error::IO(e, self.path.clone()))
    }
    
    /// Returns a slice of the [Vault] identifier.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Returns a slice of the pepper.
    pub fn pepper(&self) -> &[u8] {
        &self.pepper
    }

    /// Returns a slice of all stored [Seeds](Seed).
    pub fn seeds(&self) -> &[Seed] {
        &self.seeds
    }

    /// Inserts a new [Seed] in the back.
    pub fn push(&mut self, seed: Seed) {
        self.seeds.push(seed);
    }
    
    /// Removes [Seed] at specified index.
    pub fn remove(&mut self, seed_index: usize) {
        self.seeds.remove(seed_index);
    }
    
    /// Gets the seed at specified index.
    /// 
    /// # Errors
    /// * [`Error::SeedIndex`] if `seed_index` is out-of-bounds.
    pub fn get(&self, seed_index: usize) -> Result<&Seed> {
        self.seeds.get(seed_index).ok_or(Error::SeedIndex(seed_index))
    }

    /// Swaps seeds at specified indices.
    /// 
    /// # Errors
    /// * [`Error::SeedIndex`] if `seed_index` is out-of-bounds.
    pub fn swap(&mut self, a: usize, b: usize) -> Result<()> {
        let max = a.max(b);

        if max >= self.seeds.len() {
            Err(Error::SeedIndex(max))
        } else {
            Ok(self.seeds.swap(a, b))
        }
    }
    
    /// Extracts the password based on the given [Seed].
    /// 
    /// In order to maintain flexibility, the given key is not verified. To verify the key, first
    /// call [`Vault::verify_key`].
    pub fn password(&self, seed: &Seed, key: &str) -> String {
        generate::password(key, &self.pepper, seed)
    }
    
    /// Verifies the hash of the entered key against a hash of the key entered when the vault was
    /// created.
    pub fn verify_key(&self, key: &str) -> bool {
        generate::auth_token(key, self.pepper()) == self.auth_token
    }

    /// Calculates the path of a vault, normalizing the vault name to adhere to the POSIX portable
    /// filename standard.
    fn path_of(folder: &Path, identifier: &str) -> PathBuf {
        const EXTENSION: &str = ".vault";
        const LEGAL_SYMBOLS: &str = "._-";
        
        let file_name: String = identifier
            .ascii_chars()               // attempt to convert all non-ascii charcters
            .flatten()                   // discard characters with no known ascii representation
            .flat_map(|str| str.chars()) // iterate over all converted characters
            .filter_map(|c| {            // filter or normalize invalid path characters
                if LEGAL_SYMBOLS.contains(c) {
                    Some(c)
                } else if c.is_alphanumeric() {
                    Some(c.to_ascii_lowercase())
                } else if c == ' ' {
                    Some('_')
                } else {
                    None
                }
            })
            .take(255 - EXTENSION.len()) // enforce max length of filename
            .chain(EXTENSION.chars())    // add extension
            .collect();
        [folder, Path::new(&file_name)].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_of() {
        let data = [
            (
                ("vaults", "test"),
                "vaults/test.vault"
            ),
            (
                ("vaults", "Hello world"),
                "vaults/hello_world.vault"
            ),
            (
                ("vaults", "åäö"),
                "vaults/aao.vault"
            ),
            (
                ("vaults", "åäö"),
                "vaults/aao.vault"
            ),
            (
                ("vaults", "😀 My secret vault 😍"),
                "vaults/grinning_my_secret_vault_heart_eyes.vault"
            ),
            (
                ("vaults", "中文拉丁化"),
                "vaults/zhong_wen_la_ding_hua.vault"
            ),
        ];

        for ((folder, identifier), expected) in data {
            assert_eq!(Vault::path_of(Path::new(folder), identifier).as_path(), Path::new(expected));
        }
    }
}
