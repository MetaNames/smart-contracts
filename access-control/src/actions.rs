use pbc_contract_common::{context::ContractContext, sorted_vec_map::SortedVecMap};

use crate::{
    msg::{ACInitMsg, ACRoleMsg, ACSetAdminRoleMsg},
    state::{AccessControlState, DEFAULT_ADMIN_ROLE},
    ContractError,
};

/// ## Description
/// Initializes access control extension state
pub fn execute_init(msg: ACInitMsg) -> AccessControlState {
    let mut state = AccessControlState {
        roles: SortedVecMap::new(),
    };
    state.setup_role(DEFAULT_ADMIN_ROLE, &msg.admin_addresses);

    state
}

/// ## Description
/// Grants specified tole to specified account
/// Throws error if caller is not admin of specified role
pub fn execute_grant_role(ctx: &ContractContext, state: &mut AccessControlState, msg: ACRoleMsg) {
    assert_only_role(state, state.get_role_admin(msg.role).unwrap(), ctx);
    state.set_role(msg.role, &msg.account);
}

/// ## Description
/// Revokes specified tole from specified account
/// Throws error if caller is not admin of specified role
pub fn execute_revoke_role(ctx: &ContractContext, state: &mut AccessControlState, msg: ACRoleMsg) {
    assert_only_role(state, state.get_role_admin(msg.role).unwrap(), ctx);
    state.revoke_role(msg.role, &msg.account);
}

/// ## Description
/// Sets new tole admin for role
/// Throws error if caller is not admin of specified role
pub fn execute_set_role_admin(
    ctx: &ContractContext,
    state: &mut AccessControlState,
    msg: ACSetAdminRoleMsg,
) {
    assert_only_role(state, state.get_role_admin(msg.role).unwrap(), ctx);
    state.set_role_admin(msg.role, msg.new_admin_role);
}

/// ## Description
/// Validates that only specified role member can have access
fn assert_only_role(state: &AccessControlState, role: u8, ctx: &ContractContext) {
    assert!(
        state.has_role(role, &ctx.sender),
        "{}",
        ContractError::MissingRole
    );
}
