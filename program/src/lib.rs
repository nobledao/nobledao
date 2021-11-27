//! Title program
#![deny(missing_docs)]

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

// Export current SDK types for downstream users building with a different SDK version
pub use solana_program;
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("DG5iQsbdcPGEcCC36JXEQySyFUSW8PSR4jnch6zpTsJG");

/// Get the pubkey for the given wallet's dynastic House.
pub fn get_house_address(wallet_address: &Pubkey) -> Pubkey {
    get_house_address_and_bump_seed_internal(wallet_address, &id()).0
}

fn get_house_address_and_bump_seed_internal(
    wallet_address: &Pubkey,
    noble_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&wallet_address.to_bytes()], noble_program_id)
}

/// Get the pubkey for the given title, using the Liege title and the vassal idnex.
pub fn get_title_address(liege_address: &Pubkey, vassal_index: u8) -> Pubkey {
    get_title_address_and_bump_seed_internal(liege_address, vassal_index, &id()).0
}

fn get_title_address_and_bump_seed_internal(
    liege_address: &Pubkey,
    vassal_index: u8,
    noble_program_id: &Pubkey,
) -> (Pubkey, u8) {
    let vassal_index_seed: &[u8] = &[vassal_index; 32];
    Pubkey::find_program_address(
        &[&liege_address.to_bytes(), vassal_index_seed],
        noble_program_id,
    )
}
