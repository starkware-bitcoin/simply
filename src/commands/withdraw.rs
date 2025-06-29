use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::Args;
use elements::{Address, Txid};

use crate::{
    commands::{build::compile_program, BuildArgs},
    esplora,
    helpers::load_witness,
    script::{create_p2tr_address, unspendable_key},
    transaction::spend_script_path,
};

#[derive(Args, Debug)]
pub struct WithdrawArgs {
    #[command(flatten)]
    pub build: BuildArgs,

    /// Transaction ID to spend
    #[arg(long)]
    pub txid: String,

    /// Destination address
    #[arg(long)]
    pub destination: String,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

pub fn withdraw(args: WithdrawArgs) -> Result<()> {
    let program = compile_program(
        &args.build.entrypoint,
        Default::default(),
        false,
        args.build.mcpp_inc_path,
    )?;
    let address = create_p2tr_address(program.clone(), unspendable_key())?;

    let txid: Txid = Txid::from_str(&args.txid).map_err(|_| anyhow!("Invalid TXID format"))?;
    let (outpoint, utxo) = esplora::fetch_utxo(&txid, &address)?;

    let destination = Address::from_str(&args.destination)
        .map_err(|_| anyhow!("Invalid destination address format"))?;

    // Create and sign transaction using the transaction module
    let witness = if let Some(witness_path) = args.build.witness {
        load_witness(Some(&witness_path))?
    } else {
        Default::default()
    };
    let tx = spend_script_path(
        outpoint,
        utxo,
        destination,
        unspendable_key(),
        program,
        witness,
    )?;

    if !args.dry_run {
        let txid = esplora::broadcast_tx(tx)?;
        println!("Transaction ID: {}", txid);
    } else {
        println!("Transaction hex: {}", elements::encode::serialize_hex(&tx));
    }

    Ok(())
}
