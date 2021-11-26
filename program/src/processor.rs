//! Program state processor

use {
    crate::{
        error::RecordError,
        get_house_address_and_bump_seed_internal, get_title_address_and_bump_seed_internal,
        instruction::TitleInstruction,
        state::{HouseData, TitleData},
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
        system_instruction,
        system_program,
        sysvar::Sysvar, // for Rent::get()
    },
};

/// Instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = TitleInstruction::try_from_slice(input)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let result = match instruction {
        TitleInstruction::CreateHouse {
            coat_of_arms,
            display_name,
        } => process_create_house_account(_program_id, accounts, coat_of_arms, display_name),
        TitleInstruction::CreateTitle {
            rank,
            kind,
            required_stake_lamports,
            coat_of_arms,
            display_name,
            liege_address,
            liege_vassal_index,
        } => process_create_title_account(
            _program_id,
            accounts,
            rank,
            kind,
            required_stake_lamports,
            coat_of_arms,
            display_name,
            liege_address,
            liege_vassal_index,
        ),
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
    // Verify house address derivation, get seed for signing.
    let (house_address, bump_seed) =
        get_house_address_and_bump_seed_internal(owner_and_funder_wallet_info.key, program_id);
    if house_address != *house_account_info.key {
        msg!("Error: House address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    let house_account_signer_seeds: &[&[_]] =
        &[&owner_and_funder_wallet_info.key.to_bytes(), &[bump_seed]];

    let house_data_space = HouseData::SIZE;
    let required_lamports = rent.minimum_balance(house_data_space).max(1);

    invoke_signed(
        &system_instruction::create_account(
            owner_and_funder_wallet_info.key,
            house_account_info.key,
            required_lamports,
            house_data_space as u64,
            program_id, // owner
        ),
        &[
            owner_and_funder_wallet_info.clone(),
            house_account_info.clone(),
            system_account_info.clone(),
        ],
        &[house_account_signer_seeds],
    )?;

    {
        let dst: &mut [u8] = &mut house_account_info.data.borrow_mut();
        let house_data_struct: HouseData = HouseData {
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
    rank: u8,
    kind: u8,
    required_stake_lamports: u64,
    coat_of_arms: [u8; 128],
    display_name: [u8; 128],
    liege_address: Pubkey,
    liege_vassal_index: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let owner_and_funder_wallet_info = next_account_info(account_info_iter)?;
    let house_account_info = next_account_info(account_info_iter)?;
    let new_title_account_info = next_account_info(account_info_iter)?;
    let liege_title_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let empty_liege = liege_address == Pubkey::new(&[0; 32]);

    // Check input accounts for validity
    if !owner_and_funder_wallet_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !owner_and_funder_wallet_info.is_writable
        || !new_title_account_info.is_writable
        || (!liege_title_account_info.is_writable && !empty_liege)
    {
        msg!("No write permission for accounts: {} {} {}", owner_and_funder_wallet_info.is_writable, new_title_account_info.is_writable, liege_title_account_info.is_writable);
        return Err(ProgramError::InvalidArgument);
    }
    check_system_program(owner_and_funder_wallet_info.owner)?;

    // Check other arguments for validity
    if rank < 1 || rank > 8 {
        msg!("Invalid rank: {}", rank);
        return Err(ProgramError::InvalidArgument);
    }
    if kind < 1 || kind > 2 {
        msg!("Invalid kind: {}", rank);
        return Err(ProgramError::InvalidArgument);
    }
    if rank == 1 && !empty_liege {
        msg!("Rank 1 title must have no liege, got {}", liege_address);
        return Err(ProgramError::InvalidArgument);
    }
    if rank > 1 && empty_liege {
        msg!("Rank 2+ title must have liege, got {}", liege_address);
        return Err(ProgramError::InvalidArgument);
    }

    // Check house address matches owner/funder wallet, and get seeds for signing.
    let (house_address, bump_seed) =
        get_house_address_and_bump_seed_internal(owner_and_funder_wallet_info.key, program_id);
    if house_address != *house_account_info.key {
        msg!("Error: House address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }
    // TODO: do we need the house account to sign anything here?
    let house_account_signer_seeds: &[&[_]] =
        &[&owner_and_funder_wallet_info.key.to_bytes(), &[bump_seed]];

    // Check title address matches liege/vassal-index seeds. Get title address seeds for signing.
    let (title_address, bump_seed) = get_title_address_and_bump_seed_internal(
        liege_title_account_info.key,
        liege_vassal_index,
        program_id,
    );
    if title_address != *new_title_account_info.key {
        msg!("Error: New title address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }
    msg!("Creating title_address: {}", title_address);
    let title_account_signer_seeds: &[&[_]] = &[
        &liege_title_account_info.key.to_bytes(),
        &[liege_vassal_index; 32],
        &[bump_seed],
    ];
    // For rank 2+ titles, deserialize the liege, check that the current house holds that
    // liege title, and if so, update the vassal list.
    if rank > 1 {
        let liege_title_data: Result<TitleData, std::io::Error> = {
            let v = liege_title_account_info.data.borrow();
            let mut v_mut: &[u8] = *v;
            let r = TitleData::deserialize(&mut v_mut);
            r
        };
        match liege_title_data {
            Ok(mut td) => {
                check_authority(house_account_info, &td.holder_house_address)?;
                if td.vassal_addresses.len() != liege_vassal_index.into() {
                    msg!(
                        "Cannot add vassal #{}, liege has {} vassals",
                        liege_vassal_index,
                        td.vassal_addresses.len()
                    );
                    return Err(ProgramError::InvalidArgument);
                }
                if td.rank >= rank {
                    msg!("Rank of new title ({}) must be numerically greater than liege title ({})", rank, td.rank);
                    return Err(ProgramError::InvalidArgument);
                }
                td.vassal_addresses.push(title_address);
                td.serialize(&mut *liege_title_account_info.data.borrow_mut())?;
            }
            Err(e) => {
                msg!("couldn't deserialize liege title: {}", e);
                return Err(ProgramError::InvalidAccountData);
            }
        }
    }

    let rent = Rent::get().unwrap();
    let title_data_space = TitleData::SIZE;
    let required_lamports = rent.minimum_balance(title_data_space).max(1);

    // This will fail if the new title address already exists, which handles checking
    // that precondition for us.
    invoke_signed(
        &system_instruction::create_account(
            owner_and_funder_wallet_info.key,
            new_title_account_info.key,
            required_lamports,
            title_data_space as u64,
            program_id, // owner
        ),
        &[
            owner_and_funder_wallet_info.clone(),
            new_title_account_info.clone(),
            system_account_info.clone(),
        ],
        &[title_account_signer_seeds],
    )?;

    let title_data_struct: TitleData = TitleData {
        version: TitleData::CURRENT_VERSION,
        lifecycle_state: TitleData::INACTIVE_STATE,
        rank: rank,
        kind: kind,
        required_stake_lamports: required_stake_lamports,
        sale_price_lamports: required_stake_lamports,
        coat_of_arms: coat_of_arms,
        display_name: display_name,
        holder_house_address: *house_account_info.key,
        stake_address: Pubkey::new(&[0; 32]),
        liege_address: *liege_title_account_info.key,
        liege_vassal_index: liege_vassal_index,
        vassal_addresses: vec![],
    };
    title_data_struct
        .serialize(&mut *new_title_account_info.data.borrow_mut())
        .map_err(|e| e.into())
}

fn check_authority(authority_info: &AccountInfo, expected_authority: &Pubkey) -> ProgramResult {
    if expected_authority != authority_info.key {
        msg!(
            "Expected house {}, got house {}",
            expected_authority,
            authority_info.key
        );
        return Err(RecordError::IncorrectAuthority.into());
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
