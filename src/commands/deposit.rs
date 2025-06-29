use anyhow::Result;
use clap::Args;

use crate::{
    commands::{build::compile_program, BuildArgs},
    script::{create_p2tr_address, unspendable_key},
};

#[derive(Args, Debug)]
pub struct DepositArgs {
    #[command(flatten)]
    pub build: BuildArgs,
}

pub fn deposit(args: DepositArgs) -> Result<()> {
    let program = compile_program(
        &args.build.entrypoint,
        Default::default(),
        false,
        args.build.mcpp_inc_path,
    )?;
    let address = create_p2tr_address(program, unspendable_key())?;
    println!("P2TR address: {}", address);
    Ok(())
}
