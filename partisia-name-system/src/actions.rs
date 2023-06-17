use std::vec;

use contract_version_base::state::ContractVersionBase;
use pbc_contract_common::{
    context::ContractContext, events::EventGroup, sorted_vec_map::SortedVecMap,
};

use crate::{
    msg::{PnsInitMsg, PnsMintMsg, RecordDeleteMsg, RecordMintMsg, RecordUpdateMsg},
    state::{Domain, PartisiaNameSystemState},
    ContractError,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Inits contract state.
/// Returns [`(PartisiaNameSystemState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
pub fn execute_init(ctx: &ContractContext, msg: &PnsInitMsg) -> PartisiaNameSystemState {
    PartisiaNameSystemState {
        domains: SortedVecMap::new(),
        records: SortedVecMap::new(),
        version: ContractVersionBase::new(CONTRACT_NAME, CONTRACT_VERSION),
    }
}

/// ## Description
/// Mint a new token. Can only be executed by minter account.
/// Returns [`Vec<EventGroup>`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
pub fn execute_mint(
    ctx: &ContractContext,
    state: &mut PartisiaNameSystemState,
    msg: &PnsMintMsg,
) -> Vec<EventGroup> {
    assert!(!state.is_minted(&msg.domain), "{}", ContractError::Minted);

    // TODO: Handle parentship

    state.domains.insert(
        msg.domain.clone(),
        Domain {
            token_id: msg.token_id,
            records: SortedVecMap::new(),
            parent_id: None,
        },
    );

    vec![]
}

/// ## Description
/// Mint a new record for a domain
/// Returns [`Vec<EventGroup>`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
pub fn execute_record_mint(
    ctx: &ContractContext,
    state: &mut PartisiaNameSystemState,
    msg: &RecordMintMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(&msg.domain), "{}", ContractError::NotFound);

    let domain = state.domains.get_mut(&msg.domain).unwrap();
    domain.mint_record(&msg.class, &msg.data);

    vec![]
}

/// ## Description
/// Update a record for a domain
/// Returns [`Vec<EventGroup>`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
pub fn execute_record_update(
    ctx: &ContractContext,
    state: &mut PartisiaNameSystemState,
    msg: &RecordUpdateMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(&msg.domain), "{}", ContractError::NotFound);

    let domain = state.domains.get_mut(&msg.domain).unwrap();
    assert!(
        domain.has_record(&msg.class),
        "{}",
        ContractError::NotFound
    );

    domain.update_record_data(&msg.class, &msg.data);

    vec![]
}

/// ## Description
/// Delete a record for a domain
/// Returns [`Vec<EventGroup>`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
pub fn execute_record_delete(
    ctx: &ContractContext,
    state: &mut PartisiaNameSystemState,
    msg: &RecordDeleteMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(&msg.domain), "{}", ContractError::NotFound);

    let domain = state.domains.get_mut(&msg.domain).unwrap();
    assert!(
        domain.has_record(&msg.class),
        "{}",
        ContractError::NotFound
    );

    domain.delete_record(&msg.class);

    vec![]
}
