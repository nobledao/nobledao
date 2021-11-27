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
        coat_of_arms: String,
        /// Display name for the house. Last byte must be 0.
        display_name: String,
    },
    /// Create a new record
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` Wallet account for title creator
    /// 1. `[]` House account for title creator (will be signed by program)
    /// 2. `[writable]` New title account (will be signed by program)
    /// 3. `[writable]` Liege title account (will be signed by program)
    CreateTitle{
        /// See TitleData.rank.
        rank: u8,
        /// See TitleData.kind.
        kind: u8,
        /// Required stake for holder of this title; will also be initial sale price.
        required_stake_lamports: u64,
        /// Coat of arms URI. Last byte must be 0. Maximum length: 128.
        coat_of_arms: String,
        /// Display name for the house. Last byte must be 0. Maximum length: 128.
        display_name: String,
        /// Address of liege title. All zeroes for root title.
        liege_address: Pubkey,
        /// Index of the title into the liege's vassal vector.
        liege_vassal_index : u8,
    }
}

/// Create a new CreateHouse instruction.
pub fn create_house(
    user_wallet_address: &Pubkey,
    house_address: &Pubkey,
    coat_of_arms: String,
    display_name: String,
) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*user_wallet_address, true),
            AccountMeta::new(*house_address, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: TitleInstruction::CreateHouse {
            coat_of_arms: coat_of_arms,
            display_name: display_name,
        }
        .try_to_vec().unwrap(),
    }
}

/// Create a new CreateTitle instruction.
pub fn create_title(
    user_wallet_address: &Pubkey,
    house_address: &Pubkey,
    new_title_address: &Pubkey,
    liege_address: &Pubkey,
    rank: u8,
    kind: u8,
    required_stake_lamports: u64,
    liege_vassal_index: u8,
    coat_of_arms: String,
    display_name: String,
) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*user_wallet_address, true),
            AccountMeta::new(*house_address, false),
            AccountMeta::new(*new_title_address, false),
            AccountMeta::new(*liege_address, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: TitleInstruction::CreateTitle {
            rank: rank,
            kind: kind,
            required_stake_lamports: required_stake_lamports,
            coat_of_arms: coat_of_arms,
            display_name: display_name,
            liege_address: *liege_address,
            liege_vassal_index: liege_vassal_index,
        }
        .try_to_vec().unwrap(),
    }
}