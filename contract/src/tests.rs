use crate::state::{MintFee, MintFees};

#[test]
fn test_mint_fees() {
    let mint_fees = MintFees {
        mapping: vec![
            MintFee {
                chars_count: 1,
                gas: 200,
            },
            MintFee {
                chars_count: 2,
                gas: 150,
            },
            MintFee {
                chars_count: 3,
                gas: 100,
            },
            MintFee {
                chars_count: 4,
                gas: 50,
            },
            MintFee {
                chars_count: 5,
                gas: 5,
            },
        ],
        default_fee: 1,
    };

    let fees_tuples = [
        ("n", 200),
        ("na", 150),
        ("nam", 100),
        ("name", 50),
        ("names", 5),
        ("verylongname", 1),
    ];

    for (name, fee) in fees_tuples {
        let fees = mint_fees.get_gas_fees(name);
        assert_eq!(fees, fee);
    }
}
