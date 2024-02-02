use crate::state::{Fee, Fees};

#[test]
fn test_mint_fees() {
    let mint_fees = Fees {
        mapping: vec![
            Fee {
                chars_count: 1,
                gas: 200,
            },
            Fee {
                chars_count: 2,
                gas: 150,
            },
            Fee {
                chars_count: 3,
                gas: 100,
            },
            Fee {
                chars_count: 4,
                gas: 50,
            },
            Fee {
                chars_count: 5,
                gas: 5,
            },
        ],
        default_fee: 1,
        decimals: 6
    };

    let fees_tuples = [
        ("n", 200000000),
        ("na", 150000000),
        ("nam", 100000000),
        ("name", 50000000),
        ("names", 5000000),
        ("verylongname", 1000000),
    ];

    for (name, fee) in fees_tuples {
        let fees = mint_fees.get(name);
        assert_eq!(fees, fee);
    }
}
