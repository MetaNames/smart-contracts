use pbc_contract_common::sorted_vec_map::SortedVecMap;

use crate::{
    msg::ACInitMsg,
    state::{AccessControlBaseState, DEFAULT_ADMIN_ROLE},
};

pub fn execute_init(msg: ACInitMsg) -> AccessControlBaseState {
    let mut state = AccessControlBaseState {
        roles: SortedVecMap::new(),
    };
    let role = state.setup_role(DEFAULT_ADMIN_ROLE, &msg.admin_addresses);

    state
}
