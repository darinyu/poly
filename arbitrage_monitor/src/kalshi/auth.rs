use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use pkcs8::DecodePrivateKey;
use rsa::{
    pkcs1::DecodeRsaPrivateKey,
    pss::{BlindedSigningKey, Signature},
    signature::{RandomizedSigner, SignatureEncoding},
    RsaPrivateKey,
};
use sha2::Sha256;
use std::fs;

/// Load RSA private key from PEM file
pub fn load_private_key(path: &str) -> Result<RsaPrivateKey> {
    let pem_content = fs::read_to_string(path)
        .context(format!("Failed to read private key from {}", path))?;
    
    // Try PKCS#1 format first (RSA PRIVATE KEY)
    if let Ok(private_key) = RsaPrivateKey::from_pkcs1_pem(&pem_content) {
        return Ok(private_key);
    }
    
    // Try PKCS#8 format (PRIVATE KEY)
    let private_key = RsaPrivateKey::from_pkcs8_pem(&pem_content)
        .context("Failed to parse private key (tried both PKCS#1 and PKCS#8 formats)")?;
    
    Ok(private_key)
}

/// Generate RSA-PSS signature for Kalshi API authentication
pub fn generate_signature(
    private_key: &RsaPrivateKey,
    timestamp: u128,
    method: &str,
    path: &str,
) -> Result<String> {
    // Create message: timestamp + method + path
    let message = format!("{}{}{}", timestamp, method, path);
    
    // Create PSS signing key with SHA256
    let mut rng = rand::thread_rng();
    let signing_key = BlindedSigningKey::<Sha256>::new(private_key.clone());
    
    // Sign the message
    let signature: Signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
    
    // Base64 encode
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
    
    Ok(signature_b64)
}
