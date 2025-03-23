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

#[derive(Debug)]
pub struct AddressInfo {
    pub address: String,
    pub position: u32,
}

pub fn generate_addresses(mnemonic: &str, count: u32) -> Vec<AddressInfo> {
    // Create seed using ergo-lib's Mnemonic implementation with empty password
    let seed = Mnemonic::to_seed(mnemonic, "");
    
    // Create master key with usePre1627KeyDerivation = false (handled by ergo-lib)
    let master_key = ExtSecretKey::derive_master(seed)
        .expect("Failed to derive master key");

    let account = ChildIndexHardened::from_31_bit(0)
        .expect("Invalid account index");

    // Generate addresses for indices 0 to count-1
    (0..count)
        .map(|idx| {
            // Create path for current index: m/44'/429'/0'/0/idx
            let path = DerivationPath::new(
                account,
                vec![ChildIndexNormal::normal(idx)
                    .expect("Invalid address index")],
            );
            
            let derived_key = master_key.derive(path)
                .expect("Failed to derive key");
            
            // Get the public key and convert it to an address
            let ext_pub_key = derived_key.public_key()
                .expect("Failed to get public key");
            let address: Address = ext_pub_key.into();
            
            // Get the encoded address with mainnet prefix
            let address = AddressEncoder::encode_address_as_string(NetworkPrefix::Mainnet, &address);
            
            AddressInfo {
                address,
                position: idx,
            }
        })
        .collect()
}

pub fn generate_mnemonic(word_count: usize) -> (String, usize) {
    let (strength, actual_word_count) = if word_count == 0 {
        // If word_count is 0, randomly select one of the supported lengths
        let supported_lengths = [12, 15, 24];
        let random_index = rand::thread_rng().gen_range(0..supported_lengths.len());
        let random_word_count = supported_lengths[random_index];
        match random_word_count {
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
    
    // Use ergo-lib's mnemonic generator
    let generator = MnemonicGenerator::new(Language::English, strength);
    let mnemonic = generator.generate()
        .expect("Failed to generate mnemonic");
        
    (mnemonic, actual_word_count)
}