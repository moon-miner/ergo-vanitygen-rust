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

pub fn generate_mnemonic(word_count: usize) -> String {
    let strength = match word_count {
        12 => 128,
        24 => 256,
        _ => panic!("Unsupported word count"),
    };
    
    // Use ergo-lib's mnemonic generator
    let generator = MnemonicGenerator::new(Language::English, strength);
    generator.generate()
        .expect("Failed to generate mnemonic")
}