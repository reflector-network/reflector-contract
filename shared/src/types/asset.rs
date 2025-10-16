use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Quoted symbol descriptor
pub enum Asset {
    Stellar(Address), // Stellar asset contract address
    Other(Symbol),    // Symbol for all other external price sources
}
