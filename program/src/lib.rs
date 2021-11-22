//! Record program
#![deny(missing_docs)]

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// Export current SDK types for downstream users building with a different SDK version
pub use solana_program;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar,
};

solana_program::declare_id!("DG5iQsbdcPGEcCC36JXEQySyFUSW8PSR4jnch6zpTsJG");

/// Get the pubkey for the given wallet's dynastic House.
pub fn get_house_address(wallet_address: &Pubkey) -> Pubkey {
    get_house_address_and_bump_seed_internal(wallet_address, &id()).0
}

fn get_house_address_and_bump_seed_internal(
    wallet_address: &Pubkey,
    noble_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &wallet_address.to_bytes(),
        ],
        noble_program_id,
    )
}