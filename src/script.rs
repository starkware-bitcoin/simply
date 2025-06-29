use anyhow::anyhow;
use elements::secp256k1_zkp as secp256k1;
use elements::{
    taproot::{LeafVersion, TaprootBuilder, TaprootSpendInfo},
    Address, AddressParams, Script,
};
use simfony::CompiledProgram;

/// Create a Bitcoin script from a compiled Simplicity program
pub fn create_script(program: &CompiledProgram) -> anyhow::Result<Script> {
    let script = Script::from(program.commit().cmr().as_ref().to_vec());
    Ok(script)
}

/// Generate a (non-confidential) P2TR address from a Simfony program and a key pair
pub fn create_p2tr_address(
    program: CompiledProgram,
    x_only_public_key: secp256k1::XOnlyPublicKey,
) -> anyhow::Result<Address> {
    let script = create_script(&program)?;
    let spend_info = taproot_spending_info(script, x_only_public_key)?;

    let address = Address::p2tr(
        secp256k1::SECP256K1,
        spend_info.internal_key(),
        spend_info.merkle_root(),
        None, // TODO: use different blinding pubkey
        &AddressParams::LIQUID_TESTNET,
    );
    Ok(address)
}

/// Taproot leaf version for Simplicity (Simfony) programs
pub fn simplicity_leaf_version() -> LeafVersion {
    LeafVersion::from_u8(0xbe).expect("constant leaf version")
}

/// Unspendable key for Taproot
/// NUMS point, read more:
/// https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
pub fn unspendable_key() -> secp256k1::XOnlyPublicKey {
    secp256k1::XOnlyPublicKey::from_slice(
        &hex::decode("50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0").unwrap(),
    )
    .unwrap()
}

/// Create a TaprootSpendInfo struct for a given Simfony program and public key
pub fn taproot_spending_info(
    script: Script,
    public_key: secp256k1::XOnlyPublicKey,
) -> anyhow::Result<TaprootSpendInfo> {
    let builder = TaprootBuilder::new();
    let version = simplicity_leaf_version();

    let builder = builder
        .add_leaf_with_ver(0, script, version)
        .map_err(|e| anyhow!("Failed to add leaf to taproot builder: {}", e))?;

    let spend_info = builder
        .finalize(&secp256k1::SECP256K1, public_key)
        .map_err(|e| anyhow!("Failed to finalize taproot builder: {}", e))?;
    Ok(spend_info)
}
