//! Program instructions

use crate::id;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// Instructions supported by the program
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum TitleInstruction {
    /// Create a new record
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` Wallet account for account creator
    /// 1. `[writable]` New house account (will be signed by program)
    /// 2. `[]` System program ID
    CreateHouse{
        /// Coat of arms URI. Last byte must be 0.
        coat_of_arms: [u8; 128],
        /// Display name for the house. Last byte must be 0.
        display_name: [u8; 128],
    },
    /// Create a new record
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` Wallet account for title creator
    /// 1. `[writable]` House account for title creator (will be signed by program)
    /// 2. `[writable]` New title account (will be signed by program)
    /// 3. `[writable]` Liege title account (will be signed by program)
    CreateTitle{
        /// See TitleData.rank.
        rank: u8,
        /// See TitleData.kind.
        kind: u8,
        /// Required stake for holder of this title; will also be initial sale price.
        required_stake_lamports: u64,
        /// Coat of arms URI. Last byte must be 0.
        coat_of_arms: [u8; 128],
        /// Display name for the house. Last byte must be 0.
        display_name: [u8; 128],
        /// Address of liege title. All zeroes for root title.
        liege_address: Pubkey,
    }
}

/// Create a new CreateHouse instruction.
pub fn create_house(
    user_wallet_address: &Pubkey,
    house_address: &Pubkey,
    coat_of_arms: &[u8; 128],
    display_name: &[u8; 128],
) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*user_wallet_address, true),
            AccountMeta::new(*house_address, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: TitleInstruction::CreateHouse {
            coat_of_arms: *coat_of_arms,
            display_name: *display_name,
        }
        .try_to_vec().unwrap(),
    }
}