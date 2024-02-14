#![doc = include_str!("../README.md")]

#[macro_use]
extern crate pbc_contract_codegen;

use pbc_contract_common::address::Address;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// State of the contract
#[state]
struct ContractState {
    address: Address,
}

/// Initialize a new Nickname contract.
///
/// # Arguments
///
/// * `_ctx` - the contract context containing information about the sender and the blockchain.
///
/// # Returns
///
/// The initial state of the contract
#[init]
fn initialize(ctx: ContractContext, address: Address) -> ContractState {
    ContractState { address }
}

/// Update contract state
///
/// # Arguments
///
/// * `_ctx` - the contract context containing information about the sender and the blockchain.
/// * `state` - the current state of the contract
/// * `address` - new contract address
///
/// # Returns
/// Updated contract state
#[action(shortname = 0x01)]
fn update_address(
    _ctx: ContractContext,
    mut state: ContractState,
    address: Address,
) -> ContractState {
    state.address = address;

    state
}
