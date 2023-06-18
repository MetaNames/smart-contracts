use std::vec;

use pbc_contract_common::{
    context::ContractContext, events::EventGroup, sorted_vec_map::SortedVecMap,
};

use crate::{
    msg::{
        ApproveForAllMsg, ApproveMsg, BurnMsg, CheckOwnerMsg, InitMsg, MintMsg, MultiMintMsg,
        RevokeForAllMsg, RevokeMsg, SetBaseUriMsg, TransferFromMsg, TransferMsg, UpdateMinterMsg,
        UpdateParentMsg,
    },
    state::{MPC721ContractState, URL_LENGTH, OperatorApproval},
    ContractError,
};

/// ## Description
/// Inits contract state.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **_ctx** is an object of type [`ContractContext`]
///
/// * **msg** is an object of type [`InitMsg`]
pub fn execute_init(ctx: &ContractContext, msg: &InitMsg) -> MPC721ContractState {
    MPC721ContractState {
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        supply: 0,
        operator_approvals: vec![],
        owners: SortedVecMap::new(),
        token_approvals: SortedVecMap::new(),
        uri_template: msg.uri_template.clone(),
        token_uri_details: SortedVecMap::new(),
    }
}

/// ## Description
/// Mint a new token. Can only be executed by minter account.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`MintMsg`]
pub fn execute_mint(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &MintMsg,
) -> Vec<EventGroup> {
    assert!(!state.exists(msg.token_id), "{}", ContractError::Minted);

    let mut new_state = state;
    new_state.owners.insert(msg.token_id, ctx.sender);
    if let Some(token_uri) = msg.token_uri {
        let formatted_uri = format!("{}{}", state.uri_template, token_uri).into_bytes();
        assert!(
            formatted_uri.len() <= URL_LENGTH,
            "{}",
            ContractError::UriTooLong
        );

        let array: [u8; 128] = formatted_uri.try_into().unwrap();
        new_state.token_uri_details.insert(msg.token_id, array);
    }

    state.increase_supply();

    vec![]
}

/// Change or reaffirm the approved address for an NFT.
/// None indicates there is no approved address.
/// Throws unless `ctx.sender` is the current NFT owner, or an authorized
/// operator of the current owner.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`NFTContractState`], the current state of the contract.
///
/// * `approved`: [`Option<Address>`], The new approved NFT controller.
///
/// * `token_id`: [`u128`], The NFT to approve.
///
/// ### Returns
///
/// The new state object of type [`NFTContractState`] with an updated ledger.
pub fn execute_approve(
    ctx: ContractContext,
    mut state: MPC721ContractState,
    msg: ApproveMsg,
) -> Vec<EventGroup> {
    let owner = state.owner_of(msg.token_id);
    assert!(
        ctx.sender == owner || state.is_approved_for_all(owner, ctx.sender),
        "{}",
        ContractError::Unauthorized
    );
    state._approve(msg.approved, msg.token_id);

    vec![]
}

/// Enable or disable approval for a third party ("operator") to manage all of
/// `ctx.sender`'s assets.
/// Throws if `operator` == `ctx.sender`.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`NFTContractState`], the current state of the contract.
///
/// * `operator`: [`Address`], Address to add to the set of authorized operators.
///
/// * `approved`: [`bool`], True if the operator is approved, false to revoke approval.
///
/// ### Returns
///
/// The new state object of type [`NFTContractState`] with an updated ledger.
pub fn execute_set_approval_for_all(
    ctx: ContractContext,
    mut state: MPC721ContractState,
    msg: ApproveForAllMsg,
) -> Vec<EventGroup> {
    assert!(
        msg.operator != ctx.sender,
        "{}",
        ContractError::Unauthorized
    );

    if msg.approved {
        let already_present = state
            .operator_approvals
            .into_iter()
            .any(|approval| approval.owner == ctx.sender && approval.operator == msg.operator);
        if !already_present {
            state.operator_approvals.push(OperatorApproval {
                owner: ctx.sender,
                operator: msg.operator,
            });
        }
    } else {
        state.operator_approvals.retain(|approval| {
            !(approval.owner == ctx.sender && approval.operator == msg.operator)
        });
    }

    vec![]
}

/// ## Description
/// Only with approval extension. Transfer token from owner to spender.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`TransferFromMsg`]
pub fn execute_transfer_from(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &TransferFromMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(msg.token_id), "{}", ContractError::NotFound);

    state.transfer(&msg.from, &msg.to, msg.token_id);
    vec![]
}

/// ## Description
/// Allows spender to transfer token from the owner account.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`ApproveMsg`]
pub fn execute_approve(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &ApproveMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(msg.token_id), "{}", ContractError::NotFound);

    state.update_approvals(&ctx.sender, &msg.spender, msg.token_id, true);
    vec![]
}

/// ## Description
/// Allows operator to transfer any owner tokens from his account.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`ApproveForAllMsg`]
pub fn execute_approve_for_all(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &ApproveForAllMsg,
) -> Vec<EventGroup> {
    state.add_operator(&ctx.sender, &msg.operator);
    vec![]
}

/// ## Description
/// Remove approval.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`RevokeMsg`]
pub fn execute_revoke(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &RevokeMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(msg.token_id), "{}", ContractError::NotFound);

    state.update_approvals(&ctx.sender, &msg.spender, msg.token_id, false);
    vec![]
}

/// ## Description
/// Remove operator.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`RevokeForAllMsg`]
pub fn execute_revoke_for_all(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &RevokeForAllMsg,
) -> Vec<EventGroup> {
    state.remove_operator(&ctx.sender, &msg.operator);
    vec![]
}

/// ## Description
/// Destroy your token forever.
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`BurnMsg`]
pub fn execute_burn(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &BurnMsg,
) -> Vec<EventGroup> {
    assert!(state.is_minted(msg.token_id), "{}", ContractError::NotFound);

    state.remove_token(&ctx.sender, msg.token_id);
    state.decrease_supply();

    vec![]
}

/// ## Description
/// Check if a user owns a particular token. Will revert otherwise
/// Returns [`(MPC721ContractState, Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`CheckOwnerMsg`]
pub fn execute_ownership_check(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &CheckOwnerMsg,
) -> Vec<EventGroup> {
    let token_info = state.token_info(msg.token_id);
    match token_info {
        Some(token_info) => assert!(
            token_info.owner == msg.owner,
            "{}",
            ContractError::IncorrectOwner
        ),
        None => panic!("{}", ContractError::NotFound),
    };
    vec![]
}

/// ## Description
/// Mint Multiple NFTs in a single function call
/// Returns [` Vec<EventGroup>)`] if operation was successful,
/// otherwise panics with error message defined in [`ContractError`]
/// ## Params
/// * **ctx** is an object of type [`ContractContext`]
///
/// * **state** is an object of type [`MPC721ContractState`]
///
/// * **msg** is an object of type [`MultiMintMsg`]
pub fn execute_multi_mint(
    ctx: &ContractContext,
    state: &mut MPC721ContractState,
    msg: &MultiMintMsg,
) -> Vec<EventGroup> {
    for mint in msg.mints.iter() {
        execute_mint(ctx, state, mint);
    }

    vec![]
}
