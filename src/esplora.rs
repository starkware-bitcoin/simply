use elements::{encode, Address, OutPoint, Transaction, TxOut, Txid};

/// Fetch UTXO given the txid and vout
pub fn fetch_utxo(txid: &Txid, address: &Address) -> anyhow::Result<(OutPoint, TxOut)> {
    let url = format!(
        "https://blockstream.info/liquidtestnet/api/tx/{}/hex",
        txid.to_string()
    );
    let tx_hex = reqwest::blocking::get(&url)?.text()?;
    let tx_bytes = hex::decode(tx_hex.trim())?;
    let transaction: Transaction = encode::deserialize(&tx_bytes)?;
    for (vout, output) in transaction.output.iter().enumerate() {
        if output.script_pubkey == address.script_pubkey() {
            return Ok((OutPoint::new(*txid, vout as u32), output.clone()));
        }
    }
    Err(anyhow::anyhow!("No UTXO found for address: {}", address))
}

/// Broadcast a transaction to the network
/// Returns the txid of the broadcasted transaction
#[allow(dead_code)]
pub fn broadcast_tx(tx: Transaction) -> anyhow::Result<String> {
    let client = reqwest::blocking::Client::new();
    let txid = client
        .post("https://blockstream.info/liquidtestnet/api/tx")
        .body(encode::serialize_hex(&tx))
        .send()?
        .text()?;
    Ok(txid)
}
