use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionConfig {
    Some((Address, i128)),
    None
}