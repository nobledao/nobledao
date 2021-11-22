//! Program state
use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{program_pack::IsInitialized, pubkey::Pubkey},
};

/// Struct defining a user's House - their nobility account. A House may hold
/// 0 or more titles.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct HouseData {
    /// Struct version, allows for upgrades to the program.
    pub version: u16,

    /// Number of tokens governing this house. Immutable. Default is 1, in which case the client wallet
    /// has the authority to govern the house.
    pub governance_token_supply: u16,

    /// The URI for the coat of arms. *Mutable*. Null-terminated.
    pub coat_of_arms: [u8; 128],

    /// The Display name for this noble house. Immutable. Null-terminated.
    pub display_name: [u8; 128],

    /// Total prestige accumulated by this house. *Mutable*.
    pub prestige: i32,

    /// Total virtue accumulated by this house. *Mutable*.
    pub virtue: i32,
}

impl HouseData {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u16 = 1;
    /// Serialized size of the struct
    pub const SIZE: usize = 4 + 128 + 128 + 4 + 4;
}

impl IsInitialized for HouseData {
    /// Is initialized
    fn is_initialized(&self) -> bool {
        self.version == Self::CURRENT_VERSION && self.governance_token_supply > 0
    }
}

/// Struct defining a noble Title.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TitleData {
    /// Struct version, allows for upgrades to the program.
    pub version: u8,

    /// Lifecycle state:
    /// 0: Uninitialized
    /// 1: Created, Inactive (never sold)
    /// 2: Active
    pub lifecycle_state: u8,

    /// Title rank. Immutable. 1 == Deus (root), 2 == Emperor, 3 == King ...
    pub rank: u8,

    /// Title type. Immutable. 1 == Noble, 2 == Religious. Future types include
    /// society groups.
    pub kind: u8,

    /// Required stake, in lamports, to hold the title. Immutable. This is the
    /// also the price floor for the title.
    pub required_stake_lamports: u64,

    /// Advertised sale price, in lamports. Immutable. Anybody with this many
    /// lamports may buy the title from the current holder.
    pub sale_price_lamports: u64,

    /// The URI for the coat of arms. *Mutable*. Null-terminated.
    pub coat_of_arms: [u8; 128],

    /// Title name. Immutable. Null-terminated.
    pub display_name: [u8; 128],

    /// House address holding the title. *Mutable*. Never all zeroes.
    pub holder_house_address: Pubkey,

    /// Stake account address. Immutable. Will be all zeroes until the title is
    /// first sold.
    pub stake_address: Pubkey,

    /// Liege title address. Immutable. All zeroes if this is the root title.
    pub liege_address: Pubkey,

    /// Vassal title addresses. Mutable.
    pub vassal_addresses: Vec<Pubkey>,
}

impl TitleData {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u8 = 1;
    /// Serialized size of the struct
    pub const SIZE: usize = 2 + 1 + 1 + 8 + 8 + 128 + 128;
}

impl IsInitialized for TitleData {
    /// Is initialized
    fn is_initialized(&self) -> bool {
        self.version == Self::CURRENT_VERSION
            && self.rank != 0
            && self.kind != 0
            && self.required_stake_lamports > 0
            && self.sale_price_lamports > 0
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use solana_program::program_error::ProgramError;

    /// Version for tests
    pub const TEST_VERSION: u16 = 1;
    /// Pubkey for tests
    // pub const TEST_PUBKEY: Pubkey = Pubkey::new_from_array([100; 32]);

    pub fn test_bytes() -> Vec<u8> {
        return vec![42; 8];
    }

    #[test]
    fn serialize_data() {
        // Bytes for tests
        let TEST_BYTES: Vec<u8> = test_bytes();
        // HouseData for tests
        let TEST_RECORD_DATA: HouseData = HouseData {
            version: TEST_VERSION,
            governance_token_supply: 1,
            coat_of_arms: vec![0; 128],
            display_name: vec![0; 128],
            prestige: 10000,
            virtue: 10000,
        };
        let mut expected = vec![1, 0];
        // expected.extend_from_slice(&TEST_PUBKEY.to_bytes());
        // expected.extend_from_slice(TEST_RECORD_DATA.data.try_to_vec().unwrap().as_slice());
        assert_eq!(TEST_RECORD_DATA.try_to_vec().unwrap(), expected);
        assert_eq!(
            HouseData::try_from_slice(&expected).unwrap(),
            TEST_RECORD_DATA
        );
    }

    // #[test]
    // fn deserialize_invalid_slice() {
    //     let data = [200; Data::DATA_SIZE - 1];
    //     let mut expected = vec![TEST_VERSION];
    //     expected.extend_from_slice(&TEST_PUBKEY.to_bytes());
    //     expected.extend_from_slice(&data);
    //     let err: ProgramError = HouseData::try_from_slice(&expected).unwrap_err().into();
    //     assert!(matches!(err, ProgramError::BorshIoError(_)));
    // }
}
