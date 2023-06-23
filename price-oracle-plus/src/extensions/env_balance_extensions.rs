use soroban_sdk::{Address, Env};



use shared::types::{
    data_key::DataKey,
};

pub trait EnvBalanceExtensions {
    fn get_base_fee(&self) -> Option<i128>;

    fn set_base_fee(&self, base_fee: i128);

    fn has_sufficient_balance(&self, account: Address, amount: i128) -> bool;

    fn try_inc_balance(&self, account: Address, amount: i128) -> bool;

    fn get_balance(&self, account: Address) -> Option<i128>;
}

impl EnvBalanceExtensions for Env {
    fn get_base_fee(&self) -> Option<i128> {
        if !self.storage().has(&DataKey::BaseFee) {
            return None;
        }
        Some(self.storage().get_unchecked(&DataKey::BaseFee).unwrap())
    }

    fn set_base_fee(&self, base_fee: i128) {
        self.storage().set(&DataKey::BaseFee, &base_fee);
    }

    fn has_sufficient_balance(&self, account: Address, amount: i128) -> bool {
        let account_balance = self.get_balance(account.clone()).unwrap_or_else(|| 0);
        amount < 0 && account_balance < (amount * -1)
    }

    fn try_inc_balance(&self, account: Address, amount: i128) -> bool {
        let mut account_balance = self.get_balance(account.clone()).unwrap_or_else(|| 0);
        account_balance += amount;
        if account_balance < 0 {
            return false;
        }
        set_balance(&self, account, account_balance);
        true
    }

    fn get_balance(&self, account: Address) -> Option<i128> {
        let balance_key = DataKey::Balance(account);
        if self.storage().has(&balance_key) {
            return Some(self.storage().get_unchecked(&balance_key).unwrap());
        }
        None
    }
}

fn set_balance(e: &Env, account: Address, amount: i128) {
    e.storage().set(&DataKey::Balance(account), &amount);
}