use ergo_lib::{
    ergotree_ir::{
        chain::address::{Address, NetworkPrefix, AddressEncoder},
        serialization::{SigmaSerializable, sigma_byte_writer::SigmaByteWriter},
    },
    wallet::{
        derivation_path::{ChildIndexHardened, ChildIndexNormal, DerivationPath},
        ext_secret_key::ExtSecretKey,
        mnemonic::Mnemonic,
    },
};
use bip39;
use rand::{rngs::OsRng, RngCore};
use std::sync::OnceLock;

// Cache common objects
static WRITER_CAPACITY: usize = 128;
static PATH: OnceLock<DerivationPath> = OnceLock::new();

pub fn generate_address(mnemonic: &str) -> String {
    // Create seed using ergo-lib's Mnemonic implementation
    let seed = Mnemonic::to_seed(mnemonic, "");
    
    // Create master key with usePre1627KeyDerivation = false (handled by ergo-lib)
    let master_key = ExtSecretKey::derive_master(seed)
        .expect("Failed to derive master key");
    
    // Use cached derivation path for EIP-3: m/44'/429'/0'/0/0
    let path = PATH.get_or_init(|| {
        let mut path = DerivationPath::new(
            ChildIndexHardened::from_31_bit(44)
                .expect("Invalid purpose index"),
            vec![],
        );
        
        // Add required indices
        path = path.extend(
            ChildIndexHardened::from_31_bit(429)
                .expect("Invalid coin index")
                .into(),
        );
        path = path.extend(
            ChildIndexHardened::from_31_bit(0)
                .expect("Invalid account index")
                .into(),
        );
        path = path.extend(
            ChildIndexNormal::normal(0)
                .expect("Invalid change index")
                .into(),
        );
        path = path.extend(
            ChildIndexNormal::normal(0)
                .expect("Invalid address index")
                .into(),
        );
        
        path
    });
    
    let derived_key = master_key.derive(path.clone())
        .expect("Failed to derive key");
    
    let public_key = derived_key.public_key()
        .expect("Failed to get public key");
    
    // Create P2PK address with preallocated buffer
    let mut bytes = Vec::with_capacity(WRITER_CAPACITY);
    let mut writer = SigmaByteWriter::new(&mut bytes, None);
    public_key.public_key.sigma_serialize(&mut writer)
        .expect("Failed to serialize public key");
    
    let address = Address::p2pk_from_pk_bytes(&bytes)
        .expect("Failed to create address");
    
    // Get the encoded address with mainnet prefix
    AddressEncoder::encode_address_as_string(NetworkPrefix::Mainnet, &address)
}

pub fn generate_mnemonic(word_count: usize) -> String {
    let entropy_bytes = match word_count {
        12 => 16, // 128 bits
        24 => 32, // 256 bits
        _ => panic!("Unsupported word count"),
    };
    
    // Preallocate entropy buffer
    let mut entropy = Vec::with_capacity(entropy_bytes);
    entropy.resize(entropy_bytes, 0);
    OsRng.fill_bytes(&mut entropy);
    
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
        .expect("Failed to generate mnemonic");
    mnemonic.to_string()
}