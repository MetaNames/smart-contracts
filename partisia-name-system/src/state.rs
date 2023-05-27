use contract_version_base::state::ContractVersionBase;
use create_type_spec_derive::CreateTypeSpec;
use mpc721_hierarchy::state::{MPC721ContractState, TokenInfo};
use pbc_contract_common::{address::Address, sorted_vec_map::SortedVecMap};
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

use crate::ContractError;

/// ## Description
/// This structure describes Partisia Name System state
#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct PartisiaNameSystemState {
    pub mpc721: MPC721ContractState,
    pub version: ContractVersionBase,
    pub domains: SortedVecMap<Vec<u8>, Domain>,
    pub records: SortedVecMap<Vec<u8>, Record>,
}

#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct Domain {
    pub token_id: u128,
}

#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct Record {
    pub data: String,
}

#[repr(u8)]
#[derive(
    Eq, PartialEq, Debug, Clone, Ord, PartialOrd, Copy, CreateTypeSpec, ReadWriteState, ReadWriteRPC,
)]
pub enum RecordClass {
    /// Wallet
    #[discriminant(0)]
    Wallet {},
    /// Website
    #[discriminant(1)]
    Uri {},
    /// Twitter
    #[discriminant(2)]
    Twitter {},
}

impl PartisiaNameSystemState {
    /// ## Description
    /// Returns domain info by token id
    pub fn domain_info(&self, domain: &[u8]) -> Option<&Domain> {
        self.domains.get(&domain.to_vec())
    }

    /// ## Description
    /// Says is token id minted or not
    pub fn is_minted(&self, token_id: &[u8]) -> bool {
        self.domains.contains_key(&token_id.to_vec())
    }

    /// ## Description
    /// Returns token info by domain
    pub fn token_info(&self, domain: &[u8]) -> Option<&TokenInfo> {
        match self.domain_info(domain) {
            Some(domain) => self.mpc721.token_info(domain.token_id),
            None => None,
        }
    }

    /// ## Description
    /// This function returns token id for given domain
    pub fn token_id(&self, domain: &[u8]) -> Option<u128> {
        self.domains.get(&domain.to_vec()).map(|d| d.token_id)
    }

    /// ## Description
    /// Returns record info by token id
    pub fn record_info(&self, token_id: &[u8], class: &RecordClass) -> Option<&Record> {
        let qualified_name = Self::fully_qualified_name(token_id, class);
        self.records.get(&qualified_name)
    }

    /// ## Description
    /// Returns boolean if account is allowed to manage domain
    /// ## Params
    pub fn allowed_to_manage(&self, account: &Address, domain: &[u8]) -> bool {
        match self.domain_info(domain) {
            Some(domain) => self.mpc721.allowed_to_manage(account, domain.token_id),
            None => false,
        }
    }

    /// ## Description
    /// Mints record for token
    pub fn mint_record(&mut self, token_id: &[u8], class: &RecordClass, data: &str) {
        let record = Record {
            data: data.to_string(),
        };
        let qualified_name = Self::fully_qualified_name(token_id, class);
        assert!(
            !self.records.contains_key(&qualified_name),
            "{}",
            ContractError::RecordMinted
        );

        self.records.insert(qualified_name, record);
    }

    /// ## Description
    /// Update data of a record
    pub fn update_record_data(&mut self, token_id: &[u8], class: &RecordClass, data: &str) {
        assert!(self.is_minted(token_id), "{}", ContractError::NotMinted);

        let qualified_name = Self::fully_qualified_name(token_id, class);
        self.records.get_mut(&qualified_name).map(|record| {
            record.data = data.to_string();
            record
        });
    }

    /// ## Description
    /// Remove a record
    pub fn delete_record(&mut self, token_id: &[u8], class: &RecordClass) {
        assert!(self.is_minted(token_id), "{}", ContractError::NotMinted);

        let qualified_name = Self::fully_qualified_name(token_id, class);
        if self.records.contains_key(&qualified_name) {
            self.records.remove_entry(&qualified_name);
        } else {
            panic!("{}", ContractError::NotFound);
        }
    }

    /// ## Description
    /// Says if record minted or not
    pub fn is_record_minted(&self, token_id: &[u8], class: &RecordClass) -> bool {
        let qualified_name = Self::fully_qualified_name(token_id, class);
        self.records.contains_key(&qualified_name)
    }

    /// ## Description
    /// Get fully qualified name for token and record class.
    /// It's a vector of bytes where first byte is a class hex and the rest is a name hash
    /// ## Example
    /// 0x0 + 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
    fn fully_qualified_name(token_id: &[u8], class: &RecordClass) -> Vec<u8> {
        let class_hex: u8 = match class {
            RecordClass::Wallet {} => 0x0,
            RecordClass::Uri {} => 0x1,
            RecordClass::Twitter {} => 0x2,
        };

        let mut vec: Vec<u8> = vec![class_hex];

        vec.extend(token_id);
        vec
    }
}