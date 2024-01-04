use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
   Stellar(Address),
   Other(Symbol)
}