use crate::{
    actions::{
        action_build_mint_callback, action_build_renew_callback, action_mint,
        action_renew_subscription, PaymentIntent,
    },
    msg::{InitMsg, MintMsg, OwnerInfoEvent, RenewDomainMsg},
    state::{ContractConfig, ContractState, ContractStats, PaymentInfo, UserRole},
};

use contract_version_base::state::ContractVersionBase;
use pbc_contract_common::{
    address::Address,
    context::{CallbackContext, ContractContext},
    events::EventGroup,
};

use nft::{actions as nft_actions, msg as nft_msg};

use access_control::{actions as ac_actions, msg as ac_msg};
use airdrop::actions::{self as airdrop_actions, execute_airdrop};
use partisia_name_system::{actions as pns_actions, msg as pns_msg, state::RecordClass};
use utils::events::assert_callback_success;

use crate::ContractError;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[init]
pub fn initialize(ctx: ContractContext, msg: InitMsg) -> (ContractState, Vec<EventGroup>) {
    let payment_info = msg.config.payment_info.clone();
    assert!(
        !payment_info.is_empty(),
        "{}",
        ContractError::PaymentInfoNotValid
    );

    payment_info.into_iter().for_each(|info| {
        assert!(
            info.token.is_some(),
            "{}",
            ContractError::PaymentTokenNotSet
        );
        assert!(
            info.receiver.is_some(),
            "{}",
            ContractError::PaymentReceiverNotSet
        );
    });

    let pns = pns_actions::execute_init(&ctx);
    let nft = nft_actions::execute_init(
        &ctx,
        &nft_msg::NFTInitMsg {
            name: msg.name,
            symbol: msg.symbol,
            uri_template: msg.uri_template,
        },
    );
    let access_control = ac_actions::execute_init(&ac_msg::ACInitMsg {
        admin_addresses: msg.admin_addresses,
        additional_roles: vec![UserRole::Whitelist {} as u8, UserRole::Airdrop {} as u8],
    });
    let airdrop = airdrop_actions::execute_init();

    let state = ContractState {
        access_control,
        airdrop,
        config: msg.config,
        nft,
        pns,
        stats: ContractStats::default(),
        version: ContractVersionBase::new(CONTRACT_NAME, CONTRACT_VERSION),
    };

    (state, vec![])
}

#[action(shortname = 0x03)]
pub fn transfer_from(
    ctx: ContractContext,
    mut state: ContractState,
    from: Address,
    to: Address,
    token_id: u128,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let mut nft_events = nft_actions::execute_transfer_from(
        &ctx,
        &mut state.nft,
        &nft_msg::NFTTransferFromMsg { from, to, token_id },
    );

    let (name, _) = state.pns.get_domain_by_token_id(token_id).unwrap();
    let msg = &pns_msg::PnsRecordDeleteAllMsg { domain: name };
    let pns_events = pns_actions::execute_record_delete_all(&ctx, &mut state.pns, msg);

    nft_events.extend(pns_events);

    (state, nft_events)
}

#[action(shortname = 0x04)]
pub fn transfer_domain(
    ctx: ContractContext,
    state: ContractState,
    from: Address,
    to: Address,
    domain: String,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let token_id = state.pns.get_token_id(&domain);
    assert!(token_id.is_some(), "{}", ContractError::DomainNotMinted);

    transfer_from(ctx, state, from, to, token_id.unwrap())
}

#[action(shortname = 0x05)]
pub fn approve(
    ctx: ContractContext,
    mut state: ContractState,
    approved: Option<Address>,
    token_id: u128,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = nft_actions::execute_approve(
        &ctx,
        &mut state.nft,
        &nft_msg::NFTApproveMsg { approved, token_id },
    );

    (state, events)
}

#[action(shortname = 0x06)]
pub fn approve_domain(
    ctx: ContractContext,
    state: ContractState,
    approved: Option<Address>,
    domain: String,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    assert!(
        state.pns.is_minted(&domain),
        "{}",
        ContractError::DomainNotMinted
    );

    let token_id = state.pns.get_token_id(&domain).unwrap();

    approve(ctx, state, approved, token_id)
}

#[action(shortname = 0x07)]
pub fn set_approval_for_all(
    ctx: ContractContext,
    mut state: ContractState,
    operator: Address,
    approved: bool,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = nft_actions::execute_set_approval_for_all(
        &ctx,
        &mut state.nft,
        &nft_msg::NFTApproveForAllMsg { operator, approved },
    );

    (state, events)
}

#[allow(clippy::too_many_arguments)]
#[action(shortname = 0x09)]
pub fn mint(
    ctx: ContractContext,
    state: ContractState,
    domain: String,
    to: Address,
    payment_coin_id: u64,
    token_uri: Option<String>,
    parent_id: Option<String>,
    subscription_years: Option<u32>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    mint_domain(
        &ctx,
        state,
        &MintMsg {
            domain,
            to,
            payment_coin_id,
            token_uri,
            parent_id,
            subscription_years,
        },
    )
}

#[action(shortname = 0x10)]
pub fn mint_batch(
    ctx: ContractContext,
    state: ContractState,
    mint_msgs: Vec<MintMsg>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let mut all_events = vec![];
    let mut state_holder = state;
    for msg in mint_msgs {
        let (new_state, mint_events) = mint_domain(&ctx, state_holder, &msg);
        all_events.extend(mint_events);
        state_holder = new_state;
    }

    (state_holder, all_events)
}

/// Returns owner info as data in the event
/// the event data is of type OwnerInfoEvent
#[action(shortname = 0x11)]
pub fn owner_info(
    ctx: ContractContext,
    state: ContractState,
    address: Address,
) -> (ContractState, Vec<EventGroup>) {
    let mut event_builder = EventGroup::builder();

    let domain_count = state.nft.owners_balance.get(&address).unwrap_or(0);
    let owner_info_event = OwnerInfoEvent {
        owner: address,
        domain_count,
        total_supply: state.nft.supply,
    };

    event_builder.return_data(owner_info_event);

    (state, vec![event_builder.build()])
}

#[action(shortname = 0x12)]
pub fn is_domain_owner(
    ctx: ContractContext,
    state: ContractState,
    domain: String,
    address: Address,
) -> (ContractState, Vec<EventGroup>) {
    let token_id = state.pns.get_token_id(&domain);
    assert!(token_id.is_some(), "{}", ContractError::DomainNotMinted);

    let is_owner = state.nft.owner_of(token_id.unwrap()) == address;
    assert!(is_owner, "{}", ContractError::Unauthorized);

    (state, vec![])
}

#[action(shortname = 0x16)]
pub fn mint_custom_record_batch(
    ctx: ContractContext,
    mut state: ContractState,
    mint_msgs: Vec<pns_msg::PnsCustomRecordMintMsg>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let mut events = vec![];
    for msg in mint_msgs {
        let mint_events = pns_actions::execute_custom_record_mint(&ctx, &mut state.pns, &msg);
        events.extend(mint_events);
    }

    (state, events)
}

#[action(shortname = 0x17)]
pub fn mint_custom_record(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    key: String,
    data: Vec<u8>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = pns_actions::execute_custom_record_mint(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsCustomRecordMintMsg { domain, key, data },
    );

    (state, events)
}

#[action(shortname = 0x18)]
pub fn update_custom_record(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    key: String,
    data: Vec<u8>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = pns_actions::execute_custom_record_update(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsCustomRecordUpdateMsg { domain, key, data },
    );

    (state, events)
}

#[action(shortname = 0x19)]
pub fn delete_custom_record(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    key: String,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = pns_actions::execute_custom_record_delete(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsCustomRecordDeleteMsg { domain, key },
    );

    (state, events)
}

#[action(shortname = 0x20)]
pub fn mint_record_batch(
    ctx: ContractContext,
    mut state: ContractState,
    mint_msgs: Vec<pns_msg::PnsRecordMintMsg>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let mut events = vec![];
    for msg in mint_msgs {
        let mint_events = pns_actions::execute_record_mint(&ctx, &mut state.pns, &msg);
        events.extend(mint_events);
    }

    (state, events)
}

#[action(shortname = 0x21)]
pub fn mint_record(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    class: RecordClass,
    data: Vec<u8>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = pns_actions::execute_record_mint(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsRecordMintMsg {
            domain,
            class,
            data,
        },
    );

    (state, events)
}

#[action(shortname = 0x22)]
pub fn update_record(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    class: RecordClass,
    data: Vec<u8>,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = pns_actions::execute_record_update(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsRecordUpdateMsg {
            domain,
            class,
            data,
        },
    );

    (state, events)
}

#[action(shortname = 0x23)]
pub fn delete_record(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    class: RecordClass,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    let events = pns_actions::execute_record_delete(
        &ctx,
        &mut state.pns,
        &pns_msg::PnsRecordDeleteMsg { domain, class },
    );

    (state, events)
}

#[action(shortname = 0x24)]
pub fn update_user_role(
    ctx: ContractContext,
    mut state: ContractState,
    role: UserRole,
    address: Address,
    active: bool,
) -> (ContractState, Vec<EventGroup>) {
    if active {
        ac_actions::execute_grant_role(
            &ctx,
            &mut state.access_control,
            &ac_msg::ACRoleMsg {
                role: role as u8,
                account: address,
            },
        );
    } else {
        ac_actions::execute_revoke_role(
            &ctx,
            &mut state.access_control,
            &ac_msg::ACRoleMsg {
                role: role as u8,
                account: address,
            },
        );
    }

    (state, vec![])
}

#[action(shortname = 0x25)]
pub fn update_config(
    ctx: ContractContext,
    mut state: ContractState,
    config: ContractConfig,
) -> (ContractState, Vec<EventGroup>) {
    let is_admin = state
        .access_control
        .has_role(UserRole::Admin {} as u8, &ctx.sender);
    assert!(is_admin, "{}", ContractError::Unauthorized);

    state.config = config;

    (state, vec![])
}

#[action(shortname = 0x26)]
pub fn renew_subscription(
    ctx: ContractContext,
    mut state: ContractState,
    domain: String,
    payment_coin_id: u64,
    payer: Address,
    subscription_years: u32,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);
    assert!(
        subscription_years > 0,
        "{}",
        ContractError::InvalidSubscriptionYears
    );

    let is_admin = state
        .access_control
        .has_role(UserRole::Admin {} as u8, &ctx.sender);

    let events;
    if is_admin {
        let (new_state, renew_events) =
            action_renew_subscription(ctx, state, domain, subscription_years);

        state = new_state;
        events = renew_events;
    } else {
        let payment_info = assert_and_get_payment_info(&state.config, payment_coin_id);
        let total_fees = payment_info.fees.get(&domain) * subscription_years as u128;
        events = action_build_renew_callback(
            &PaymentIntent {
                id: payment_coin_id,
                receiver: payment_info.receiver.unwrap(),
                token: payment_info.token.unwrap(),
                total_fees,
            },
            &RenewDomainMsg {
                domain,
                payment_coin_id,
                payer,
                subscription_years,
            },
            0x31,
        );
    };

    (state, events)
}

#[action(shortname = 0x27)]
pub fn add_airdrop(
    ctx: ContractContext,
    mut state: ContractState,
    addresses: Vec<Address>,
) -> (ContractState, Vec<EventGroup>) {
    assert_has_role(&state, UserRole::Airdrop {}, &ctx.sender);

    for address in addresses {
        airdrop_actions::execute_add_airdrop(&mut state.airdrop, &address);
    }

    (state, vec![])
}

#[callback(shortname = 0x30)]
pub fn on_mint_callback(
    ctx: ContractContext,
    callback_ctx: CallbackContext,
    state: ContractState,
    msg: MintMsg,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    assert_callback_success(&callback_ctx);

    assert_and_get_payment_info(&state.config, msg.payment_coin_id);

    action_mint(
        &ctx,
        state,
        &msg.domain,
        &msg.to,
        &msg.token_uri,
        &msg.parent_id,
        &msg.subscription_years,
    )
}

#[callback(shortname = 0x31)]
pub fn on_renew_subscription_callback(
    ctx: ContractContext,
    callback_ctx: CallbackContext,
    state: ContractState,
    msg: RenewDomainMsg,
) -> (ContractState, Vec<EventGroup>) {
    assert_contract_enabled(&state);

    assert_callback_success(&callback_ctx);

    assert_and_get_payment_info(&state.config, msg.payment_coin_id);

    action_renew_subscription(ctx, state, msg.domain, msg.subscription_years)
}

fn mint_domain(
    ctx: &ContractContext,
    state: ContractState,
    mint_msg: &MintMsg,
) -> (ContractState, Vec<EventGroup>) {
    let MintMsg {
        domain,
        to,
        payment_coin_id,
        token_uri,
        parent_id,
        subscription_years,
    } = mint_msg;

    assert!(!state.pns.is_minted(domain), "{}", ContractError::Minted);

    pns_actions::validate_domain(domain);

    let mut events = vec![];
    let mut mut_state = state;

    let is_admin = mut_state
        .access_control
        .has_role(UserRole::Admin {} as u8, &ctx.sender);
    if parent_id.is_some() || is_admin {
        let (new_state, mint_events) = action_mint(
            ctx,
            mut_state,
            domain,
            to,
            token_uri,
            parent_id,
            subscription_years,
        );

        mut_state = new_state;

        events.extend(mint_events);
    } else {
        let config = &mut_state.config;
        if config.whitelist_enabled {
            let is_whitelisted = mut_state
                .access_control
                .has_role(UserRole::Whitelist {} as u8, &ctx.sender);
            assert!(is_whitelisted, "{}", ContractError::UserNotWhitelisted);
        }

        if config.mint_count_limit_enabled && !is_admin {
            let mint_count = mut_state.stats.mint_count.get(&ctx.sender);
            assert!(
                mint_count.is_none() || mint_count <= Some(config.mint_count_limit),
                "{}",
                ContractError::MintCountLimitReached
            );
        }

        let has_airdrop = mut_state.airdrop.has_airdrop(&ctx.sender);
        if has_airdrop {
            let domain_length = domain.chars().count() as u32;
            assert!(
                domain_length >= config.airdrop_min_domain_length,
                "{}",
                ContractError::AirdropNotValid
            );

            execute_airdrop(&mut mut_state.airdrop, &ctx.sender);

            let (new_state, mint_events) = action_mint(
                ctx,
                mut_state,
                domain,
                to,
                token_uri,
                parent_id,
                subscription_years,
            );

            mut_state = new_state;

            events.extend(mint_events);
        } else {
            let payment_info = assert_and_get_payment_info(config, *payment_coin_id);
            let subscription_years = subscription_years.unwrap_or(1);
            let total_fees = payment_info.fees.get(domain) * subscription_years as u128;
            let payout_transfer_events = action_build_mint_callback(
                &PaymentIntent {
                    id: *payment_coin_id,
                    receiver: payment_info.receiver.unwrap(),
                    token: payment_info.token.unwrap(),
                    total_fees,
                },
                &MintMsg {
                    domain: domain.to_string(),
                    to: *to,
                    payment_coin_id: *payment_coin_id,
                    token_uri: token_uri.clone(),
                    parent_id: parent_id.clone(),
                    subscription_years: Some(subscription_years),
                },
                0x30,
            );

            events.extend(payout_transfer_events);
        }
    }

    (mut_state, events)
}

fn assert_has_role(state: &ContractState, role: UserRole, account: &Address) {
    assert!(
        state.access_control.has_role(role as u8, account),
        "{}",
        ContractError::Unauthorized
    );
}

fn assert_contract_enabled(state: &ContractState) {
    assert!(
        state.config.contract_enabled,
        "{}",
        ContractError::ContractDisabled
    );
}

fn assert_and_get_payment_info(config: &ContractConfig, payment_coin_id: u64) -> PaymentInfo {
    let payment_info = config.get_payment_info(payment_coin_id);
    assert!(
        payment_info.is_some(),
        "{}",
        ContractError::PaymentInfoNotValid
    );

    payment_info.unwrap()
}
