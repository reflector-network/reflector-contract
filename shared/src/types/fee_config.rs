use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Oracle retention config containing fee asset and daily retention fee amount
pub enum FeeConfig {
    Some((Address, i128)),
    None,
}
