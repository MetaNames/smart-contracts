use utils::tests::{mock_address, mock_contract_context};

use crate::state::{AccessControlState, DEFAULT_ADMIN_ROLE};

const ROLE_A: u8 = 0x02;
const ROLE_B: u8 = 0x03;

#[test]
fn proper_access_control() {
    let alice = mock_address(1u8);
    let bob = mock_address(2u8);
    let jack = mock_address(3u8);

    let mut access_control = AccessControlState::default();

    assert!(!access_control.has_role(ROLE_A, &alice));
    assert_eq!(access_control.get_role_admin(ROLE_A), None);

    access_control.setup_role(DEFAULT_ADMIN_ROLE, &alice);
    assert!(!access_control.has_role(ROLE_A, &alice));
    assert!(access_control.has_role(DEFAULT_ADMIN_ROLE, &alice));

    assert!(!access_control.has_role(ROLE_B, &bob));
    access_control.setup_role(ROLE_B, &bob);
    assert!(access_control.has_role(ROLE_B, &bob));

    assert!(!access_control.has_role(ROLE_B, &jack));
    access_control.grant_role(ROLE_B, &jack, &mock_contract_context(1u8));
    assert!(access_control.has_role(ROLE_B, &jack));

    assert_eq!(access_control.get_role_admin(ROLE_B), Some(0x00));

    access_control.assert_only_role(ROLE_B, &mock_contract_context(3u8));

    assert!(access_control.has_role(ROLE_B, &jack));
    access_control.revoke_role(ROLE_B, &jack, &mock_contract_context(1u8));
    assert!(!access_control.has_role(ROLE_B, &jack));

    access_control.setup_role(ROLE_A, &bob);
    access_control.set_role_admin(ROLE_A, ROLE_B);

    assert_eq!(access_control.get_role_admin(ROLE_A), Some(0x03));

    access_control.renounce_role(DEFAULT_ADMIN_ROLE, &mock_contract_context(1u8));
    assert!(!access_control.has_role(DEFAULT_ADMIN_ROLE, &alice));
}

#[test]
#[should_panic(expected = "AccessControl-base: Specified address is missing role")]
fn test_role_mismatch() {
    let jack = mock_address(3u8);

    let mut access_control = AccessControlState::default();

    access_control.setup_role(DEFAULT_ADMIN_ROLE, &jack);

    access_control.assert_only_role(ROLE_A, &mock_contract_context(3u8));
}
