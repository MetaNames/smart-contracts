use std::{collections::BTreeMap};

use contract_version_base::state::ContractVersionBase;
use create_type_spec_derive::CreateTypeSpec;
use mpc721_hierarchy::state::{MPC721ContractState, TokenInfo};
use pbc_contract_common::address::Address;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

use crate::ContractError;

/// ## Description
/// This structure describes Partisia Name System state
#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct PartisiaNameSystemState {
    pub mpc721: MPC721ContractState,
    pub version: ContractVersionBase,
    /// the domain key is the domain name in bytes
    pub domains: BTreeMap<Vec<u8>, Domain>,
}

#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct Domain {
    pub token_id: u128,
    pub records: BTreeMap<RecordClass, Record>,
    pub parent: Option<Vec<u8>>,
}

#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct Record {
    pub data: Vec<u8>,
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

impl Domain {
    /// ## Description
    /// Returns record info by class
    pub fn record_info(&self, class: &RecordClass) -> Option<&Record> {
        self.records.get(class)
    }

    /// ## Description
    /// Returns record data given record class
    pub fn record_data(&self, class: &RecordClass) -> Option<&Vec<u8>> {
        let record = self.record_info(class);
        if record.is_none() {
            return None;
        }

        Some(&record.unwrap().data)
    }

    /// ## Description
    /// Says if record class is minted or not
    pub fn is_record_minted(&self, class: &RecordClass) -> bool {
        self.records.contains_key(class)
    }

    /// ## Description
    /// Mints record given record class
    pub fn mint_record(&mut self, class: &RecordClass, data: &Vec<u8>) {
        let record = Record { data: data.clone() };
        assert!(
            !self.is_record_minted(class),
            "{}",
            ContractError::RecordMinted
        );

        self.records.insert(class.clone(), record);
    }

    /// ## Description
    /// Update record data given record class
    pub fn update_record_data(&mut self, class: &RecordClass, data: &Vec<u8>) {
        assert!(
            self.is_record_minted(class),
            "{}",
            ContractError::RecordNotMinted
        );

        self.records.get_mut(class).unwrap().data = data.clone();
    }

    /// ## Description
    /// Remove record given record class
    pub fn remove_record(&mut self, class: &RecordClass) {
        assert!(
            self.is_record_minted(class),
            "{}",
            ContractError::RecordNotMinted
        );

        self.records.remove(class);
    }
}

impl PartisiaNameSystemState {
    /// ## Description
    /// Returns domain info by token id
    pub fn domain_info(&self, domain: &[u8]) -> Option<&Domain> {
        self.domains.get(domain)
    }

    /// ## Description
    /// Says is token id minted or not
    pub fn is_minted(&self, domain: &[u8]) -> bool {
        self.domains.contains_key(domain)
    }

    /// ## Description
    /// Returns token info by domain
    pub fn token_info(&self, domain: &[u8]) -> Option<&TokenInfo> {
        let domain = self.domain_info(domain);
        if domain.is_none() {
            return None;
        }

        self.mpc721.token_info(domain.unwrap().token_id)
    }

    /// ## Description
    /// This function returns token id for given domain
    pub fn token_id(&self, domain: &[u8]) -> Option<u128> {
        self.domains.get(domain).map(|d| d.token_id)
    }

    /// ## Description
    /// Returns boolean if account is allowed to manage domain
    /// ## Params
    pub fn allowed_to_manage(&self, account: &Address, domain: &[u8]) -> bool {
        let domain = self.domain_info(domain);
        if domain.is_none() {
            return false;
        }

        self.mpc721
            .allowed_to_manage(account, domain.unwrap().token_id)
    }
}
