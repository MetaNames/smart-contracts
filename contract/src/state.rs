use access_control::state::AccessControlState;
use contract_version_base::state::ContractVersionBase;
use create_type_spec_derive::CreateTypeSpec;
use nft::state::NFTContractState;
use partisia_name_system::state::PartisiaNameSystemState;
use pbc_contract_common::{address::Address, sorted_vec_map::SortedVecMap};
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

#[allow(unused_imports)]
use crate::contract::__PBC_IS_ZK_CONTRACT;

#[state]
#[derive(PartialEq, Eq, Default, Clone, Debug)]
pub struct ContractState {
    pub access_control: AccessControlState,
    pub config: ContractConfig,
    pub nft: NFTContractState,
    pub pns: PartisiaNameSystemState,
    pub stats: ContractStats,
    pub version: ContractVersionBase,
}

#[repr(C)]
#[derive(
    ReadWriteRPC, ReadWriteState, CreateTypeSpec, PartialEq, Eq, Default, Copy, Clone, Debug,
)]
pub struct PayableMintInfo {
    // Those are required but need to be optional for Default trait to work
    pub id: u64,
    pub token: Option<Address>,
    pub receiver: Option<Address>,
}

#[repr(u8)]
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, PartialEq, Eq, Copy, Clone, Debug)]
pub enum UserRole {
    #[discriminant(0)]
    Admin {},
    #[discriminant(1)]
    Whitelist {},
}


#[repr(C)]
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, PartialEq, Eq, Default, Clone, Debug)]
pub struct MintFee {
    pub chars_count: u32,
    pub gas: u128,
}

#[repr(C)]
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, PartialEq, Eq, Default, Clone, Debug)]
pub struct MintFees {
    pub mapping: Vec<MintFee>,
    pub default_fee: u128,
}

#[repr(C)]
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, PartialEq, Eq, Default, Clone, Debug)]
pub struct ContractConfig {
    pub contract_enabled: bool,
    pub mint_count_limit_enabled: bool,
    pub mint_count_limit: u32,
    pub payable_mint_info: Vec<PayableMintInfo>,
    pub whitelist_enabled: bool,
    pub mint_fees: MintFees,
}

#[repr(C)]
#[derive(ReadWriteState, CreateTypeSpec, PartialEq, Eq, Default, Clone, Debug)]
pub struct ContractStats {
    pub mint_count: SortedVecMap<Address, u32>,
}

impl ContractConfig {
    pub fn get_payable_mint_info(&self, id: u64) -> Option<PayableMintInfo> {
        for info in &self.payable_mint_info {
            if info.id == id {
                return Some(*info);
            }
        }

        None
    }
}

impl ContractStats {
    pub fn increase_mint_count(&mut self, address: Address) {
        let count = self.mint_count.get(&address).unwrap_or(&0);
        self.mint_count.insert(address, count + 1);
    }
}

impl MintFees {
    pub fn get_gas_fees(&self, domain: &str) -> u128 {
        let chars_count = domain.len() as u32;
        for fee in &self.mapping {
            if fee.chars_count == chars_count {
                return fee.gas;
            }
        }

        self.default_fee
    }
}
