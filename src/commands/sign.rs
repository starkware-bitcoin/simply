use anyhow::{Context, Result};
use clap::Args;
use elements::hashes::sha256;
use elements::hashes::Hash as _;
use elements::secp256k1_zkp::{
    rand::rngs::OsRng, Keypair, Message, Secp256k1, SecretKey, XOnlyPublicKey,
};
use hex::FromHex;

#[derive(Args, Debug)]
pub struct SignArgs {
    /// Message to sign, hex-encoded
    #[arg(long, value_name = "HEX")]
    pub message: String,

    /// Secret key, hex-encoded (32 bytes). If omitted, a random key is generated
    #[arg(long, value_name = "HEX")]
    pub secret: Option<String>,
}

pub fn sign(args: SignArgs) -> Result<()> {
    // Decode message from hex and hash to 32 bytes for BIP340
    let msg_bytes = Vec::from_hex(&args.message).with_context(|| "Failed to decode message hex")?;
    anyhow::ensure!(!msg_bytes.is_empty(), "Message must not be empty");

    let digest = sha256::Hash::hash(&msg_bytes);
    let msg = Message::from_digest_slice(digest.as_ref())
        .with_context(|| "Failed to construct message for signing")?;

    // Obtain or generate secret key
    let (secret_key, generated): (SecretKey, bool) = if let Some(sk_hex) = args.secret {
        let sk_bytes = <[u8; 32]>::from_hex(sk_hex.trim())
            .with_context(|| "Failed to decode secret key hex (expected 32 bytes)")?;
        let sk = SecretKey::from_slice(&sk_bytes).with_context(|| "Invalid secret key")?;
        (sk, false)
    } else {
        let mut rng = OsRng;
        (SecretKey::new(&mut rng), true)
    };

    let secp = Secp256k1::new();
    let keypair = Keypair::from_secret_key(&secp, &secret_key);
    let (xonly_pub, _parity): (XOnlyPublicKey, _) = XOnlyPublicKey::from_keypair(&keypair);

    // BIP340 Schnorr signature
    let sig = secp.sign_schnorr(&msg, &keypair);

    // Outputs
    println!("Signature (BIP340): {}", hex::encode(sig.as_ref()));
    println!("Message: {}", args.message);
    println!(
        "Public key (x-only): {}",
        hex::encode(xonly_pub.serialize())
    );
    if generated {
        println!("Private key: {}", hex::encode(secret_key.secret_bytes()));
    }

    Ok(())
}
