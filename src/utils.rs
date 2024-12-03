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
use std::sync::OnceLock;

// Cache common objects
static PATH: OnceLock<DerivationPath> = OnceLock::new();

pub fn generate_address(mnemonic: &str) -> String {
    // Create seed using ergo-lib's Mnemonic implementation with empty password
    let seed = Mnemonic::to_seed(mnemonic, "");
    
    // Create master key with usePre1627KeyDerivation = false (handled by ergo-lib)
    let master_key = ExtSecretKey::derive_master(seed)
        .expect("Failed to derive master key");
    
    // Use cached derivation path for EIP-3: m/44'/429'/0'/0/0
    let path = PATH.get_or_init(|| {
        // Create path with account index 0' and address index 0
        // The purpose (44') and coin type (429') are added automatically
        DerivationPath::new(
            ChildIndexHardened::from_31_bit(0)
                .expect("Invalid account index"),
            vec![ChildIndexNormal::normal(0)
                .expect("Invalid address index")],
        )
    });
    
    let derived_key = master_key.derive(path.clone())
        .expect("Failed to derive key");
    
    // Get the public key and convert it to an address
    let ext_pub_key = derived_key.public_key()
        .expect("Failed to get public key");
    let address: Address = ext_pub_key.into();
    
    // Get the encoded address with mainnet prefix
    AddressEncoder::encode_address_as_string(NetworkPrefix::Mainnet, &address)
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