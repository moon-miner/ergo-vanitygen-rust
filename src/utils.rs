use ergo_lib::{
    ergotree_ir::{
        chain::address::{Address, NetworkPrefix, AddressEncoder},
    },
    wallet::{
        derivation_path::{ChildIndexHardened, ChildIndexNormal, DerivationPath},
        ext_secret_key::ExtSecretKey,
        mnemonic::Mnemonic,
        mnemonic_generator::{Language, MnemonicGenerator},
    },
};
use rand::Rng;
use std::ops::{Deref, Drop};

/// Represents an address along with its derivation position.
#[derive(Debug)]
pub struct AddressInfo {
    pub address: String,
    pub position: u32,
}

/// A secure container for sensitive seed phrases that will be zeroed out when dropped
#[derive(Clone)]
pub struct SecureSeed {
    data: Vec<u8>,
}

impl SecureSeed {
    /// Create a new secure seed from a string
    pub fn new(seed_phrase: &str) -> Self {
        Self {
            data: seed_phrase.as_bytes().to_vec(),
        }
    }
    
    /// Get the seed phrase as a string reference
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.data).unwrap_or_default()
    }
    
    /// Intentionally expose the seed phrase and take ownership
    pub fn expose(self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

impl Deref for SecureSeed {
    type Target = [u8];
    
    fn deref(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for SecureSeed {
    fn drop(&mut self) {
        // Zero out the memory to prevent leaving sensitive data in memory
        for byte in self.data.iter_mut() {
            *byte = 0;
        }
    }
}

/// Generates a list of addresses from a given mnemonic.
/// 
/// The function derives a master key from the mnemonic, then generates `count` addresses
/// using the derivation path m/44'/429'/0'/0/idx. It returns a vector of `AddressInfo`.
pub fn generate_addresses(mnemonic: &str, count: u32) -> Vec<AddressInfo> {
    // Create the seed from the mnemonic with an empty password.
    let seed = Mnemonic::to_seed(mnemonic, "");

    // Derive the master key.
    let master_key = ExtSecretKey::derive_master(seed)
        .expect("Failed to derive master key");

    // Use account index 0 (hardened).
    let account = ChildIndexHardened::from_31_bit(0)
        .expect("Invalid account index");

    // Generate addresses for indices 0 to count - 1.
    (0..count)
        .map(|idx| {
            // Build derivation path: m/44'/429'/0'/0/idx
            let path = DerivationPath::new(
                account,
                vec![ChildIndexNormal::normal(idx)
                    .expect("Invalid address index")],
            );

            // Derive the key for the given path.
            let derived_key = master_key.derive(path)
                .expect("Failed to derive key");

            // Convert the derived public key to an address.
            let ext_pub_key = derived_key.public_key()
                .expect("Failed to get public key");
            let address: Address = ext_pub_key.into();

            // Encode the address with Mainnet prefix.
            let encoded_address = AddressEncoder::encode_address_as_string(NetworkPrefix::Mainnet, &address);

            AddressInfo {
                address: encoded_address,
                position: idx,
            }
        })
        .collect()
}

/// Generates a mnemonic phrase and returns it wrapped in a SecureSeed along with its actual word count.
/// 
/// If `word_count` is 0, a supported length is chosen at random (12, 15, or 24 words).
/// Otherwise, only 12, 15, or 24 are allowed.
pub fn generate_secure_mnemonic(word_count: usize) -> (SecureSeed, usize) {
    let (strength, actual_word_count) = if word_count == 0 {
        let supported_lengths = [12, 15, 24];
        let random_index = rand::thread_rng().gen_range(0..supported_lengths.len());
        match supported_lengths[random_index] {
            12 => (128, 12),
            15 => (160, 15),
            24 => (256, 24),
            _ => unreachable!(),
        }
    } else {
        match word_count {
            12 => (128, 12),
            15 => (160, 15),
            24 => (256, 24),
            _ => panic!("Unsupported word count"),
        }
    };

    let generator = MnemonicGenerator::new(Language::English, strength);
    let mnemonic = generator.generate()
        .expect("Failed to generate mnemonic");

    (SecureSeed::new(&mnemonic), actual_word_count)
}
