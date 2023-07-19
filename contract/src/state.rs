use contract_version_base::state::ContractVersionBase;
use create_type_spec_derive::CreateTypeSpec;
use nft::state::NFTContractState;
use partisia_name_system::state::PartisiaNameSystemState;
use pbc_contract_common::address::Address;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

#[allow(unused_imports)]
use crate::contract::__PBC_IS_ZK_CONTRACT;

#[state]
#[derive(PartialEq, Eq, Default, Clone, Debug)]
pub struct ContractState {
    pub pns: PartisiaNameSystemState,
    pub nft: NFTContractState,
    pub payable_mint_info: PayableMintInfo,
    pub version: ContractVersionBase,
}

#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, PartialEq, Eq, Default, Clone, Debug)]
pub struct PayableMintInfo {
    // It's required but need to be optional for Default trait to work
    pub token: Option<Address>,
    // TODO: Calculate amount dynamically
    pub amount: u128,
}
