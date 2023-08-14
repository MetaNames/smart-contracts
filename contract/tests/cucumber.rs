use std::{panic::catch_unwind, time::Duration};

use cucumber::{given, then, when, World};
use meta_names_contract::{
    contract::{
        approve_domain, initialize, mint, on_mint_callback, renew_subscription, transfer_domain,
        update_config, update_user_role,
    },
    msg::{InitMsg, MintMsg},
    state::{ContractConfig, ContractState, PayableMintInfo, UserRole},
};
use partisia_name_system::{
    actions::{execute_record_mint, execute_record_update, execute_update_expiration},
    msg::{PnsDomainUpdateExpirationMsg, PnsRecordMintMsg, PnsRecordUpdateMsg},
    state::RecordClass,
};
use utils::{
    tests::{mock_address, mock_contract_context, mock_successful_callback_context},
    time::{duration_in_years, unix_epoch_now_as_duration},
};

const SYSTEM_ADDRESS: u8 = 0;
const ALICE_ADDRESS: u8 = 1;
const BOB_ADDRESS: u8 = 2;
const PAYABLE_TOKEN_ADDRESS: u8 = 10;

#[derive(Debug, Default, World)]
pub struct ContractWorld {
    state: ContractState,
    point_in_time: Duration,
}

fn get_address_for_user(user: String) -> u8 {
    match user.as_str() {
        "Alice" => ALICE_ADDRESS,
        "Bob" => BOB_ADDRESS,
        "contract" => SYSTEM_ADDRESS,
        _ => panic!("Unknown user"),
    }
}

fn get_user_role(role: String) -> UserRole {
    match role.as_str() {
        "admin" => UserRole::Admin {},
        "whitelist" => UserRole::Whitelist {},
        _ => panic!("Unknown role"),
    }
}

// Taken from partisia-name-system/tests/cucumber.rs
fn get_record_class_given(class: String) -> RecordClass {
    match class.as_str() {
        "Bio" => RecordClass::Bio {},
        "Discord" => RecordClass::Discord {},
        "Uri" => RecordClass::Uri {},
        "Twitter" => RecordClass::Twitter {},
        "Wallet" => RecordClass::Wallet {},
        "Custom" => RecordClass::Custom {},
        "Custom2" => RecordClass::Custom2 {},
        "Custom3" => RecordClass::Custom3 {},
        "Custom4" => RecordClass::Custom4 {},
        "Custom5" => RecordClass::Custom5 {},
        _ => panic!("Unknown record class"),
    }
}

#[given(regex = "a meta names contract")]
fn meta_names_contract(world: &mut ContractWorld) {
    let config = ContractConfig {
        contract_enabled: true,
        payable_mint_info: PayableMintInfo {
            token: Some(mock_address(PAYABLE_TOKEN_ADDRESS)),
            receiver: Some(mock_address(ALICE_ADDRESS)),
        },
        ..ContractConfig::default()
    };

    let msg = InitMsg {
        admin_addresses: vec![mock_address(SYSTEM_ADDRESS)],
        config,
        name: "Meta Names".to_string(),
        symbol: "META".to_string(),
        uri_template: "metanames.io".to_string(),
    };

    let (state, _) = initialize(mock_contract_context(ALICE_ADDRESS), msg);
    world.state = state;
    world.point_in_time = unix_epoch_now_as_duration();
}

#[given(regex = r"(contract) config '(.+)' is '(.+)'")]
#[when(regex = r"(\w+) updates the config '(.+)' to '(.+)'")]
fn update_contract_config(world: &mut ContractWorld, user: String, key: String, value: String) {
    let res = catch_unwind(|| {
        let new_config = match key.as_str() {
            "contract_enabled" => {
                let mut new_config = world.state.config.clone();
                new_config.contract_enabled = value == "true";
                new_config
            }
            "whitelist_enabled" => {
                let mut new_config = world.state.config.clone();
                new_config.whitelist_enabled = value == "true";
                new_config
            }
            "mint_count_limit_enabled" => {
                let mut new_config = world.state.config.clone();
                new_config.mint_count_limit_enabled = value == "true";
                new_config
            }
            "mint_count_limit" => {
                let mut new_config = world.state.config.clone();
                new_config.mint_count_limit = value.parse::<u32>().unwrap();
                new_config
            }
            _ => panic!("Unknown config key"),
        };

        update_config(
            mock_contract_context(get_address_for_user(user.clone())),
            world.state.clone(),
            new_config,
        )
    });

    if let Ok((new_state, _)) = res {
        world.state = new_state;
    }
}

#[given(expr = "{word} minted '{word}' domain without a parent")]
#[when(expr = "{word} mints '{word}' domain without fees and a parent")]
fn mint_a_domain(world: &mut ContractWorld, user: String, domain: String) {
    let res = catch_unwind(|| {
        on_mint_callback(
            mock_contract_context(get_address_for_user(user.clone())),
            mock_successful_callback_context(),
            world.state.clone(),
            MintMsg {
                domain,
                to: mock_address(get_address_for_user(user)),
                token_uri: None,
                parent_id: None,
                subscription_years: None,
            },
        )
    });

    if let Ok((new_state, _)) = res {
        world.state = new_state;
    }
}

#[given(regex = r"(\w+) user (with) the (\w+) (role)")]
#[when(regex = r"(\w+) user (grants|denies) the (\w+) role for (\w+) user")]
fn user_admin_role(
    world: &mut ContractWorld,
    admin: String,
    action: String,
    role: String,
    user: String,
) {
    let res = catch_unwind(|| match action.as_str() {
        "with" => update_user_role(
            mock_contract_context(SYSTEM_ADDRESS),
            world.state.clone(),
            get_user_role(role),
            mock_address(get_address_for_user(admin)),
            true,
        ),
        "grants" | "denied" => update_user_role(
            mock_contract_context(get_address_for_user(admin)),
            world.state.clone(),
            get_user_role(role),
            mock_address(get_address_for_user(user)),
            action == "grants",
        ),
        _ => panic!("Unknown action"),
    });

    if let Ok((new_state, _)) = res {
        world.state = new_state;
    }
}

#[given(expr = "{word} approved {word} on '{word}' domain")]
fn user_approve_domain(world: &mut ContractWorld, user: String, approved: String, domain: String) {
    let (new_state, _) = approve_domain(
        mock_contract_context(get_address_for_user(user)),
        world.state.clone(),
        Some(mock_address(get_address_for_user(approved))),
        domain,
    );

    world.state = new_state;
}

// Taken from partisia-name-system/tests/cucumber.rs
#[given(regex = ".+ (minted) the '(.+)' record with '(.+)' data for the '(.+)' domain")]
#[when(regex = ".+ (mints|updates) the '(.+)' record with '(.+)' data for the '(.+)' domain")]
fn mint_a_record(
    world: &mut ContractWorld,
    action: String,
    class: String,
    data: String,
    domain: String,
) {
    let res = catch_unwind(|| {
        let mut state = world.state.clone();
        let context = &mock_contract_context(1);
        match action.as_str() {
            "mints" | "minted" => {
                let msg = PnsRecordMintMsg {
                    domain,
                    class: get_record_class_given(class),
                    data: data.into_bytes(),
                };
                execute_record_mint(context, &mut state.pns, &msg);
            }

            "updates" => {
                let msg = PnsRecordUpdateMsg {
                    domain,
                    class: get_record_class_given(class),
                    data: data.into_bytes(),
                };

                execute_record_update(context, &mut state.pns, &msg);
            }

            _ => panic!("Not handled"),
        };

        state
    });

    if let Ok(new_state) = res {
        world.state = new_state;
    }
}

#[given(expr = "{word} renewed '{word}' domain for {int} years")]
#[when(expr = "{word} renews '{word}' domain for {int} years")]
fn renew_domain(world: &mut ContractWorld, user: String, domain_name: String, years: u32) {
    let context = mock_contract_context(get_address_for_user(user.clone()));

    // To properly test renewing a domain, we need to override the expiration time of the domain
    let expires_at = Some(world.point_in_time.as_secs() as i64);
    execute_update_expiration(
        &context,
        &mut world.state.pns,
        &PnsDomainUpdateExpirationMsg {
            domain: domain_name.clone(),
            expires_at,
        },
    );

    let res = catch_unwind(|| {
        renew_subscription(
            context,
            world.state.clone(),
            domain_name,
            mock_address(get_address_for_user(user)),
            years,
        )
    });

    if let Ok((new_state, _)) = res {
        world.state = new_state;
    }
}

#[when(expr = "{word} mints '{word}' domain with '{word}' domain as the parent")]
#[when(regex = r"(\w+) mints '(.+)' domain without (a parent)")]
fn mint_domain_with_parent(
    world: &mut ContractWorld,
    user: String,
    domain: String,
    parent: String,
) {
    let res = catch_unwind(|| {
        let parent_opt = if parent == "a parent" {
            None
        } else {
            Some(parent.clone())
        };

        mint(
            mock_contract_context(get_address_for_user(user.clone())),
            world.state.clone(),
            domain,
            mock_address(get_address_for_user(user)),
            None,
            parent_opt,
            Some(1),
        )
    });

    if let Ok((new_state, _)) = res {
        world.state = new_state;
    }
}

#[when(expr = "{word} transfers the '{word}' domain to {word}")]
fn transfer_domain_to(world: &mut ContractWorld, user: String, domain: String, to: String) {
    let res = catch_unwind(|| {
        transfer_domain(
            mock_contract_context(get_address_for_user(user.clone())),
            world.state.clone(),
            mock_address(get_address_for_user(user.clone())),
            mock_address(get_address_for_user(to)),
            domain,
        )
    });

    if let Ok((new_state, _)) = res {
        world.state = new_state;
    }
}

#[then(expr = "{word} owns '{word}' domain")]
fn owns_the_domain(world: &mut ContractWorld, user: String, domain: String) {
    let domain = world.state.pns.get_domain(&domain).unwrap();

    assert_eq!(
        world.state.nft.owner_of(domain.token_id),
        mock_address(get_address_for_user(user))
    );
}

#[then(expr = "'{word}' domain is not minted")]
fn domain_is_not_minted(world: &mut ContractWorld, domain: String) {
    let domain = world.state.pns.get_domain(&domain);

    assert_eq!(domain, None);
}

#[then(expr = "{word} mint count is {int}")]
fn mint_counts(world: &mut ContractWorld, user: String, count: u32) {
    let user = mock_address(get_address_for_user(user));

    assert_eq!(world.state.stats.mint_count.get(&user), Some(&count));
}

#[then(regex = r"(\w+) user (has|has not) the (\w+) role")]
fn user_is_admin(world: &mut ContractWorld, user: String, has: String, role: String) {
    let has_role = world.state.access_control.has_role(
        get_user_role(role) as u8,
        &mock_address(get_address_for_user(user)),
    );

    assert_eq!(has_role, has == "has");
}

#[then(regex = "the contract config '(.+)' is '(.+)'")]
fn contract_config_is(world: &mut ContractWorld, key: String, value: String) {
    let config = world.state.config.clone();

    match key.as_str() {
        "whitelist_enabled" => assert_eq!(config.whitelist_enabled, value == "true"),
        "mint_count_limit_enabled" => assert_eq!(config.mint_count_limit_enabled, value == "true"),
        "mint_count_limit" => {
            let value = value.parse::<u32>().unwrap();
            assert_eq!(config.mint_count_limit, value);
        }
        _ => panic!("Unknown config key"),
    }
}

// Taken from partisia-name-system/tests/cucumber.rs
#[then(expr = "'{word}' domain does not have a '{word}' record")]
fn domain_has_no_record(world: &mut ContractWorld, domain: String, class: String) {
    let domain = world.state.pns.get_domain(&domain);

    if let Some(domain) = domain {
        let record = domain.get_record(&get_record_class_given(class));

        assert_eq!(record, None);
    }
}

#[then(expr = "'{word}' domain expires in {int} years")]
fn domain_expires_in(world: &mut ContractWorld, domain: String, years: u32) {
    let domain = world.state.pns.get_domain(&domain).unwrap();

    let expected_expires_at = world.point_in_time + duration_in_years(years as u64);
    assert_eq!(
        domain.expires_at,
        Some(expected_expires_at.as_secs() as i64)
    );
}

// This runs before everything else, so you can setup things here.
fn main() {
    // You may choose any executor you like (`tokio`, `async-std`, etc.).
    // You may even have an `async` main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    futures::executor::block_on(ContractWorld::run("tests/features"));
}
