use contract_version_base::state::ContractVersionBase;
use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::sorted_vec_map::SortedVecMap;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

use crate::ContractError;

pub const MAX_RECORD_DATA_LENGTH: usize = 64;
pub const MAX_DOMAIN_LEN: usize = 32;

/// ## Description
/// This structure describes Partisia Name System state
#[derive(ReadWriteState, CreateTypeSpec, Clone, Default, PartialEq, Eq, Debug)]
pub struct PartisiaNameSystemState {
    pub version: ContractVersionBase,
    pub domains: SortedVecMap<String, Domain>,
}

#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub enum Domain {
    #[discriminant(0)]
    Parent {
        token_id: u128,
        minted_at: i64,
        expires_at: i64,
        records: SortedVecMap<RecordClass, Record>,
    },
    #[discriminant(1)]
    Child {
        token_id: u128,
        parent_id: String,
        minted_at: i64,
        records: SortedVecMap<RecordClass, Record>,
    },
}

#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct Record {
    pub data: Vec<u8>,
}

#[repr(u8)]
#[derive(
    Eq, PartialEq, Debug, Clone, Ord, PartialOrd, Copy, CreateTypeSpec, ReadWriteState, ReadWriteRPC,
)]
pub enum RecordClass {
    #[discriminant(0)]
    Bio {},
    #[discriminant(1)]
    Discord {},
    #[discriminant(2)]
    Twitter {},
    #[discriminant(3)]
    Uri {},
    #[discriminant(4)]
    Wallet {},
    #[discriminant(5)]
    Avatar {},
    // Customizables
    #[discriminant(6)]
    Custom {},
    #[discriminant(7)]
    Custom2 {},
    #[discriminant(8)]
    Custom3 {},
    #[discriminant(9)]
    Custom4 {},
    #[discriminant(10)]
    Custom5 {},
}

impl Domain {
    /// Get domain token id
    pub fn get_token_id(&self) -> u128 {
        match self {
            &Domain::Child {
                token_id,
                parent_id,
                minted_at,
                records,
            } => token_id,
            &Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                records,
            } => token_id,
        }
    }

    ///  Get domain minted at
    pub fn get_minted_at(&self) -> i64 {
        match self {
            &Domain::Child {
                token_id,
                parent_id,
                minted_at,
                records,
            } => minted_at,
            &Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                records,
            } => minted_at,
        }
    }

    /// ## Description
    /// Get record given class
    pub fn get_record(&self, class: &RecordClass) -> Option<&Record> {
        match self {
            &Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                records,
            } => records.get(class),
            &Domain::Child {
                token_id,
                parent_id,
                minted_at,
                records,
            } => records.get(class),
        }
    }

    /// ## Description
    /// Existence of record given class
    pub fn is_record_minted(&self, class: &RecordClass) -> bool {
        match self {
            &Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                records,
            } => records.contains_key(class),
            &Domain::Child {
                token_id,
                parent_id,
                minted_at,
                records,
            } => records.contains_key(class),
        }
    }

    /// ## Description
    /// Checks if domain is active
    /// Opposite of expired
    pub fn is_active(&self, unix_millis_now: i64) -> bool {
        match self {
            &Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                records,
            } => expires_at >= unix_millis_now,
            &Domain::Child {
                token_id,
                parent_id,
                minted_at,
                records,
            } => true,
        }
    }

    /// ## Description
    /// Mints record for token
    pub fn mint_record(&mut self, class: &RecordClass, data: &[u8]) {
        assert!(
            !self.is_record_minted(class),
            "{}",
            ContractError::RecordMinted
        );

        let record = Record {
            data: data.to_vec(),
        };

        match self {
            &mut Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                mut records,
            } => records.insert(*class, record),
            &mut Domain::Child {
                token_id,
                parent_id,
                minted_at,
                mut records,
            } => records.insert(*class, record),
        };
    }

    /// ## Description
    /// Update data of a record
    pub fn update_record_data(&mut self, class: &RecordClass, data: &[u8]) {
        assert!(
            self.is_record_minted(class),
            "{}",
            ContractError::RecordNotMinted
        );

        match self {
            &mut Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                mut records,
            } => records.get_mut(class).map(|record| {
                record.data = data.to_vec();
                record
            }),
            &mut Domain::Child {
                token_id,
                parent_id,
                minted_at,
                mut records,
            } => records.get_mut(class).map(|record| {
                record.data = data.to_vec();
                record
            }),
        };
    }

    /// ## Description
    /// Remove a record
    pub fn delete_record(&mut self, class: &RecordClass) {
        assert!(
            self.is_record_minted(class),
            "{}",
            ContractError::RecordNotMinted
        );

        match self {
            &mut Domain::Parent {
                token_id,
                minted_at,
                expires_at,
                mut records,
            } => {
                if records.contains_key(class) {
                    records.remove_entry(class);
                } else {
                    panic!("{}", ContractError::NotFound);
                }
            }
            &mut Domain::Child {
                token_id,
                parent_id,
                minted_at,
                mut records,
            } => {
                if records.contains_key(class) {
                    records.remove_entry(class);
                } else {
                    panic!("{}", ContractError::NotFound);
                }
            }
        };
    }
}

impl PartisiaNameSystemState {
    /// ## Description
    /// Returns info given domain
    pub fn get_domain(&self, domain_name: &str) -> Option<&Domain> {
        self.domains.get(&String::from(domain_name))
    }

    /// ## Description
    /// Returns if the domain is active
    /// If the domain is a subdomain, it checks if the parent is active
    pub fn is_active(&self, domain_name: &str, unix_millis_now: i64) -> bool {
        match self.get_domain(domain_name) {
            Some(domain) => {
                domain.is_active(unix_millis_now)
                    && self
                        .get_root_parent(domain_name)
                        .map_or(true, |parent| parent.is_active(unix_millis_now))
            }
            None => false,
        }
    }

    pub fn get_domain_by_token_id(&self, token_id: u128) -> Option<(&String, &Domain)> {
        self.domains
            .iter()
            .find(|(_, domain)| domain.get_token_id() == token_id)
    }

    /// ## Description
    /// Returns parent info by domain
    pub fn get_parent(&self, domain: &Domain) -> Option<&Domain> {
        match domain {
            Domain::Child {
                token_id,
                parent_id,
                minted_at,
                records,
            } => {
                if !self.domains.contains_key(parent_id) {
                    panic!("Expected parent domain not found")
                }

                self.domains.get(parent_id)
            }
            _ => None,
        }
    }

    /// Get all parents of a domain
    pub fn get_parents(&self, domain_name: &str) -> Vec<&Domain> {
        let mut parents: Vec<&Domain> = vec![];
        let mut current_domain = self.get_domain(domain_name);

        while let Some(domain) = current_domain {
            if let Some(parent) = self.get_parent(domain) {
                parents.push(parent);
                current_domain = Some(parent);
            } else {
                current_domain = None;
            }
        }

        parents
    }

    /// Get root parent of a domain
    pub fn get_root_parent(&self, domain_name: &str) -> Option<&Domain> {
        let parents = self.get_parents(domain_name);

        match parents.last() {
            Some(parent) => {
                // By definition, the root parent has no parent
                match parent {
                    Domain::Parent {
                        token_id,
                        minted_at,
                        expires_at,
                        records,
                    } => Some(parent),
                    _ => {
                        panic!("Expected root parent to have no parent")
                    }
                }
            }
            None => None,
        }
    }

    /// ## Description
    /// Says is token id minted or not
    pub fn is_minted(&self, domain_name: &str) -> bool {
        self.domains.contains_key(&String::from(domain_name))
    }

    /// ## Description
    /// This function returns token id for given domain
    pub fn get_token_id(&self, domain_name: &str) -> Option<u128> {
        self.domains
            .get(&String::from(domain_name))
            .map(|d| d.get_token_id())
    }
}
