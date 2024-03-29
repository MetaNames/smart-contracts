use std::{mem::take, panic::catch_unwind};

use cucumber::{given, then, when, World};
use partisia_name_system::{
    actions::{
        execute_init, execute_mint, execute_record_delete, execute_record_delete_all,
        execute_record_mint, execute_record_update,
    },
    msg::{
        PnsMintMsg, PnsRecordDeleteAllMsg, PnsRecordDeleteMsg, PnsRecordMintMsg, PnsRecordUpdateMsg,
    },
    state::{PartisiaNameSystemState, RecordClass},
};
use utils::tests::{mock_contract_context, tomorrow_timestamp, yesterday_timestamp};

fn get_record_class_given(class: String) -> RecordClass {
    match class.as_str() {
        "Bio" => RecordClass::Bio {},
        "Discord" => RecordClass::Discord {},
        "Uri" => RecordClass::Uri {},
        "Twitter" => RecordClass::Twitter {},
        "Wallet" => RecordClass::Wallet {},
        "Avatar" => RecordClass::Avatar {},
        "Custom" => RecordClass::Custom {},
        "Custom2" => RecordClass::Custom2 {},
        "Custom3" => RecordClass::Custom3 {},
        "Custom4" => RecordClass::Custom4 {},
        "Custom5" => RecordClass::Custom5 {},
        _ => panic!("Unknown record class"),
    }
}

#[derive(Debug, Default, World)]
pub struct PartisiaNameSystemWorld {
    state: PartisiaNameSystemState,
}

#[given("a PNS contract")]
fn pns_contract(world: &mut PartisiaNameSystemWorld) {
    let state = execute_init(&mock_contract_context(1));

    world.state = state;
}

#[given(regex = "'(.+)' domain is expired")]
fn domain_is_expired(world: &mut PartisiaNameSystemWorld, domain_name: String) {
    let mut domain = world.state.domains.get(&domain_name).unwrap();
    domain.expires_at = Some(yesterday_timestamp());
    world.state.domains.insert(domain_name, domain);
}

#[given(regex = ".+ minted '(.+)' domain without a parent")]
#[when(regex = ".+ mints '(.+)' domain without a parent")]
fn mint_a_domain(world: &mut PartisiaNameSystemWorld, domain: String) {
    let msg = PnsMintMsg {
        domain,
        token_id: 0,
        parent_id: None,
        expires_at: Some(tomorrow_timestamp()),
    };

    let res = catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut state = take(&mut world.state);
        execute_mint(&mock_contract_context(1), &mut state, &msg);
        state
    }));

    if let Ok(new_state) = res {
        world.state = new_state;
    }
}

#[given(regex = ".+ minted '(.+)' domain with '(.+)' domain as the parent")]
#[when(regex = ".+ mints '(.+)' domain with '(.+)' domain as the parent")]
fn mint_a_domain_with_parent(world: &mut PartisiaNameSystemWorld, domain: String, parent: String) {
    let msg = PnsMintMsg {
        domain,
        token_id: 0,
        parent_id: Some(parent),
        expires_at: None,
    };

    let res = catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut state = take(&mut world.state);
        execute_mint(&mock_contract_context(1), &mut state, &msg);
        state
    }));

    if let Ok(new_state) = res {
        world.state = new_state;
    }
}

#[given(regex = ".+ (minted) the '(.+)' record with '(.+)' data for the '(.+)' domain")]
#[when(regex = ".+ (mints|updates) the '(.+)' record with '(.+)' data for the '(.+)' domain")]
fn mint_a_record(
    world: &mut PartisiaNameSystemWorld,
    action: String,
    class: String,
    data: String,
    domain: String,
) {
    let res = catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut state = take(&mut world.state);
        let context = &mock_contract_context(1);
        match action.as_str() {
            "mints" | "minted" => {
                let msg = PnsRecordMintMsg {
                    domain,
                    class: get_record_class_given(class),
                    data: data.into_bytes(),
                };
                execute_record_mint(context, &mut state, &msg);
            }

            "updates" => {
                let msg = PnsRecordUpdateMsg {
                    domain,
                    class: get_record_class_given(class),
                    data: data.into_bytes(),
                };

                execute_record_update(context, &mut state, &msg);
            }

            _ => panic!("Not handled"),
        };

        state
    }));

    if let Ok(new_state) = res {
        world.state = new_state;
    }
}

#[when(regex = ".+ deletes the '(.+)' record for the '(.+)' domain")]
fn domain_record_delete(world: &mut PartisiaNameSystemWorld, class: String, domain: String) {
    let msg = PnsRecordDeleteMsg {
        domain,
        class: get_record_class_given(class),
    };

    execute_record_delete(&mock_contract_context(1), &mut world.state, &msg);
}

#[when(regex = ".+ deletes all records for the '(.+)' domain")]
fn domain_record_delete_all(world: &mut PartisiaNameSystemWorld, domain: String) {
    let msg = PnsRecordDeleteAllMsg { domain };

    execute_record_delete_all(&mock_contract_context(1), &mut world.state, &msg);
}

#[then(regex = "'(.+)' domain (is|is not) minted")]
fn is_domain_minted(world: &mut PartisiaNameSystemWorld, domain: String, action: String) {
    let domain = world.state.get_domain(&domain);

    match action.as_str() {
        "is" => assert!(domain.is_some()),
        "is not" => assert!(domain.is_none()),
        _ => panic!("Not handled"),
    }
}

#[then(expr = "'{word}' domain has a '{word}' record with '{word}' data")]
fn domain_has_record(
    world: &mut PartisiaNameSystemWorld,
    domain: String,
    class: String,
    data: String,
) {
    let domain = world.state.get_domain(&domain);

    if let Some(domain) = domain {
        let record = domain.get_record(&get_record_class_given(class)).unwrap();

        assert_eq!(*record, data.into_bytes());
    }
}

#[then(expr = "'{word}' domain does not exist")]
fn has_no_domain(world: &mut PartisiaNameSystemWorld, domain: String) {
    let domain = world.state.get_domain(&domain);

    assert_eq!(domain, None);
}

#[then(expr = "'{word}' domain does not have a '{word}' record")]
fn domain_has_no_record(world: &mut PartisiaNameSystemWorld, domain: String, class: String) {
    let domain = world.state.get_domain(&domain);

    if let Some(domain) = domain {
        let record = domain.get_record(&get_record_class_given(class));

        assert_eq!(record, None);
    }
}

fn main() {
    futures::executor::block_on(PartisiaNameSystemWorld::run("tests/features"));
}
