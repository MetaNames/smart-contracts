use pbc_contract_common::sorted_vec_map::SortedVecMap;

use crate::{
    msg::ACInitMsg,
    state::{AccessControlState, DEFAULT_ADMIN_ROLE},
};

pub fn execute_init(msg: ACInitMsg) -> AccessControlState {
    let mut state = AccessControlState {
        roles: SortedVecMap::new(),
    };
    state.setup_role(DEFAULT_ADMIN_ROLE, &msg.admin_addresses);

    state
}
