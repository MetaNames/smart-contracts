use nft::{actions as nft_actions, msg as nft_msg};
use partisia_name_system::{actions as pns_actions, msg as pns_msg};
use pbc_contract_common::{address::Address, context::ContractContext, events::EventGroup};
use std::time::Duration;

use crate::{
    msg::{MPC20TransferFromMsg, MintMsg, RenewDomainMsg},
    state::{ContractState, PayableMintInfo},
    ContractError,
};
use utils::{
    events::{build_msg_callback, IntoShortnameRPCEvent},
    time::{duration_in_years_of, unix_epoch_now_as_duration},
};

/// Action to mint contract
pub fn action_mint(
    ctx: ContractContext,
    state: ContractState,
    domain: String,
    to: Address,
    token_uri: Option<String>,
    parent_id: Option<String>,
    subscription_years: Option<u32>,
) -> (ContractState, Vec<EventGroup>) {
    assert!(!state.pns.is_minted(&domain), "{}", ContractError::Minted);

    pns_actions::validate_domain(&domain);

    let mut expires_at: Option<i64> = None;

    // Parent validations
    if let Some(parent_id) = parent_id.clone() {
        let parent = state.pns.get_domain(&parent_id);
        assert!(parent.is_some(), "{}", ContractError::DomainNotMinted);

        let parent = parent.unwrap();
        assert!(parent.is_active(), "{}", ContractError::DomainNotActive);

        pns_actions::validate_domain_with_parent(&domain, &parent_id);

        let parent_token_id = parent.token_id;
        assert!(
            state.nft.is_approved_or_owner(ctx.sender, parent_token_id),
            "{}",
            ContractError::Unauthorized
        );
    } else if let Some(years_active) = subscription_years {
        let date = unix_epoch_now_as_duration() + duration_in_years_of(years_active as u64);
        expires_at = Some(date.as_secs() as i64);
    }

    let mut state = state;
    let token_id = state.nft.get_next_token_id();
    let nft_events = nft_actions::execute_mint(
        &ctx,
        &mut state.nft,
        &nft_msg::NFTMintMsg {
            to,
            token_id,
            token_uri,
        },
    );

    let pns_events = pns_actions::execute_mint(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsMintMsg {
            domain,
            expires_at,
            parent_id,
            token_id,
        },
    );

    state.stats.increate_mint_count(ctx.sender);

    let events = nft_events
        .into_iter()
        .chain(pns_events.into_iter())
        .collect();

    (state, events)
}

pub fn action_build_mint_callback(
    ctx: ContractContext,
    payable_mint_info: PayableMintInfo,
    mint_msg: &MintMsg,
    callback_byte: u32,
) -> Vec<EventGroup> {
    let mut payout_transfer_events = EventGroup::builder();

    let subscription_years = mint_msg.subscription_years.unwrap_or(1);
    MPC20TransferFromMsg {
        from: mint_msg.to,
        to: payable_mint_info.receiver.unwrap(),
        amount: calculate_mint_fees(mint_msg.domain.as_str(), subscription_years),
    }
    .as_interaction(
        &mut payout_transfer_events,
        &payable_mint_info.token.unwrap(),
    );

    build_msg_callback(&mut payout_transfer_events, callback_byte, mint_msg);

    vec![payout_transfer_events.build()]
}

pub fn action_build_renew_callback(
    ctx: ContractContext,
    payable_mint_info: PayableMintInfo,
    renew_msg: &RenewDomainMsg,
    callback_byte: u32,
) -> Vec<EventGroup> {
    let mut payout_transfer_events = EventGroup::builder();

    MPC20TransferFromMsg {
        from: renew_msg.payer,
        to: payable_mint_info.receiver.unwrap(),
        amount: calculate_mint_fees(renew_msg.domain.as_str(), renew_msg.subscription_years),
    }
    .as_interaction(
        &mut payout_transfer_events,
        &payable_mint_info.token.unwrap(),
    );

    build_msg_callback(&mut payout_transfer_events, callback_byte, renew_msg);

    vec![payout_transfer_events.build()]
}

pub fn action_renew_subscription(
    ctx: ContractContext,
    state: ContractState,
    domain: String,
    subscription_years: u32,
) -> (ContractState, Vec<EventGroup>) {
    let mut state = state;
    let mut domain = state.pns.get_mut_domain(&domain).unwrap();

    let mut new_expiration_at = match domain.expires_at {
        Some(expires_at) => parse_timestamp_as_duration(expires_at),
        None => unix_epoch_now_as_duration(),
    };
    new_expiration_at += duration_in_years_of(subscription_years as u64);

    domain.expires_at = Some(new_expiration_at.as_secs() as i64);

    (state, vec![])
}

pub fn calculate_mint_fees(domain_name: &str, years: u32) -> u128 {
    let length = domain_name.len();
    let amount = match length {
        1 => 200,
        2 => 150,
        3 => 100,
        4 => 50,
        _ => 5,
    };

    amount * years as u128
}

fn parse_timestamp_as_duration(timestamp: i64) -> Duration {
    Duration::from_secs(timestamp as u64)
}
