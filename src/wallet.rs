use pqcrypto_dilithium::dilithium5::*;
use pqcrypto_traits::sign::{
    PublicKey as PublicKeyTrait, 
    SecretKey as SecretKeyTrait, 
    DetachedSignature as DetachedSignatureTrait
};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Clone, Serialize, Deserialize)]
pub struct QuantumWallet {
    pub public_key: Box<[u8]>,
    pub secret_key: Box<[u8]>,
    pub address: String,
}

impl QuantumWallet {
    pub fn new(filename: &str) -> Self {
        if let Ok(mut file) = File::open(filename) {
            let mut data = Vec::new();
            if file.read_to_end(&mut data).is_ok() {
                if let Ok(wallet) = serde_json::from_slice::<QuantumWallet>(&data) {
                    println!("🔐 Himera Wallet Loaded: {}", wallet.address);
                    return wallet;
                }
            }
        }

        println!("🆕 Generating Dilithium5 Quantum Keys...");
        let (pk, sk) = keypair();
        let address_hash = hex::encode(&pk.as_bytes()[..16]);
        
        let wallet = QuantumWallet {
            public_key: pk.as_bytes().to_vec().into_boxed_slice(),
            secret_key: sk.as_bytes().to_vec().into_boxed_slice(),
            address: format!("HIM_{}", address_hash),
        };

        let serialized = serde_json::to_vec(&wallet).unwrap();
        let mut file = File::create(filename).expect("Failed to create wallet file");
        file.write_all(&serialized).expect("Failed to write wallet data");
        
        println!("✅ New Quantum Wallet Created: {}", wallet.address);
        wallet
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk = SecretKey::from_bytes(&self.secret_key).expect("SK Load Error");
        let sig = detached_sign(message, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(message: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> bool {
        match PublicKey::from_bytes(public_key_bytes) {
            Ok(pk) => {
                match DetachedSignature::from_bytes(signature_bytes) {
                    Ok(sig) => verify_detached_signature(&sig, message, &pk).is_ok(),
                    Err(_) => false,
                }
            },
            Err(_) => false,
        }
    }
}
