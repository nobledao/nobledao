//! Program state processor

use {
    crate::{
        error::RecordError,
        instruction::TitleInstruction,
        state::HouseData,
        get_house_address_and_bump_seed_internal,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::IsInitialized,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,  // for Rent::get()
        system_instruction,
        system_program,
    },
};

fn check_authority(authority_info: &AccountInfo, expected_authority: &Pubkey) -> ProgramResult {
    if expected_authority != authority_info.key {
        msg!("Incorrect record authority provided");
        return Err(RecordError::IncorrectAuthority.into());
    }
    if !authority_info.is_signer {
        msg!("Record authority signature missing");
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = TitleInstruction::try_from_slice(input)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let result = match instruction {
        TitleInstruction::CreateHouse { coat_of_arms, display_name } => {
            process_create_house_account(_program_id, accounts, coat_of_arms, display_name)
        }
        TitleInstruction::CreateTitle {
            rank,
            kind,
            required_stake_lamports,
            coat_of_arms,
            display_name,
            liege_address
        } => {
            Ok(())
        }
    };
    result
}

/// Processes CreateHouse instruction
pub fn process_create_house_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    coat_of_arms: [u8; 128],
    display_name: [u8; 128],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let owner_and_funder_wallet_info = next_account_info(account_info_iter)?;
    let house_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    if !owner_and_funder_wallet_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !owner_and_funder_wallet_info.is_writable || !house_account_info.is_writable {
        return Err(ProgramError::InvalidArgument);
    }
    check_system_program(owner_and_funder_wallet_info.owner)?;

    let rent = Rent::get().unwrap();

    let (house_address, bump_seed) = get_house_address_and_bump_seed_internal(
        owner_and_funder_wallet_info.key,
        program_id,
    );
    if house_address != *house_account_info.key {
        msg!("Error: House address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    let house_account_signer_seeds: &[&[_]] = &[
        &owner_and_funder_wallet_info.key.to_bytes(),
        &[bump_seed],
    ];

    let house_data_space = HouseData::SIZE;
    let required_lamports = rent.minimum_balance(house_data_space).max(1);

    invoke_signed(
        &system_instruction::create_account(
            owner_and_funder_wallet_info.key,
            house_account_info.key,
            required_lamports,
            house_data_space as u64,
            program_id,  // owner
        ),
        &[
            owner_and_funder_wallet_info.clone(),
            house_account_info.clone(),
            system_account_info.clone(),
        ],
        &[house_account_signer_seeds],
    )?;

    {
        let dst : &mut [u8] = &mut house_account_info.data.borrow_mut();
        let house_data_struct : HouseData = HouseData{
            version: HouseData::CURRENT_VERSION,
            governance_token_supply: 1,
            coat_of_arms: coat_of_arms,
            display_name: display_name,
            prestige: 0,
            virtue: 0,
        };
        let data = house_data_struct.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }

    Ok(())
}

/// Processes CreateTitle instruction
pub fn process_create_title_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    coat_of_arms: [u8; 128],
    display_name: [u8; 128],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let owner_and_funder_wallet_info = next_account_info(account_info_iter)?;
    let house_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    if !owner_and_funder_wallet_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !owner_and_funder_wallet_info.is_writable || !house_account_info.is_writable {
        return Err(ProgramError::InvalidArgument);
    }
    check_system_program(owner_and_funder_wallet_info.owner)?;

    let rent = Rent::get().unwrap();

    let (house_address, bump_seed) = get_house_address_and_bump_seed_internal(
        owner_and_funder_wallet_info.key,
        program_id,
    );
    if house_address != *house_account_info.key {
        msg!("Error: House address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    let house_account_signer_seeds: &[&[_]] = &[
        &owner_and_funder_wallet_info.key.to_bytes(),
        &[bump_seed],
    ];

    let house_data_space = HouseData::SIZE;
    let required_lamports = rent.minimum_balance(house_data_space).max(1);

    invoke_signed(
        &system_instruction::create_account(
            owner_and_funder_wallet_info.key,
            house_account_info.key,
            required_lamports,
            house_data_space as u64,
            program_id,  // owner
        ),
        &[
            owner_and_funder_wallet_info.clone(),
            house_account_info.clone(),
            system_account_info.clone(),
        ],
        &[house_account_signer_seeds],
    )?;

    {
        let dst : &mut [u8] = &mut house_account_info.data.borrow_mut();
        let house_data_struct : HouseData = HouseData{
            version: HouseData::CURRENT_VERSION,
            governance_token_supply: 1,
            coat_of_arms: coat_of_arms,
            display_name: display_name,
            prestige: 0,
            virtue: 0,
        };
        let data = house_data_struct.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }

    Ok(())
}


/// Check system program address
fn check_system_program(program_id: &Pubkey) -> Result<(), ProgramError> {
    if *program_id != system_program::id() {
        msg!(
            "Expected system program {}, received {}",
            system_program::id(),
            program_id
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}