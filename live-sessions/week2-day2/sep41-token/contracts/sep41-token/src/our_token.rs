use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Env, IntoVal, MuxedAddress, String,
};

use crate::{
    error::ContractError,
    events::{Approval, Transfer},
    storage::{AllowanceKey, DataKey},
    token_trait::TokenInterface,
};

#[contract]
pub struct SibToken;

#[contractimpl]
impl SibToken {
    pub fn __constructor(env: Env, admin: Address, initial_supply: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("The contract is already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);

        if initial_supply > 0 {
            let current = Self::balance(env.clone(), admin.clone());
            env.storage()
                .persistent()
                .set(&DataKey::Balance(admin), &(current + initial_supply));
        }
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(AllowanceKey { from, spender }))
            .unwrap_or(0)
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        live_until_ledger: u32,
    ) {
        from.require_auth();

        let from_balance = Self::balance(env.clone(), from.clone());

        if from_balance < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        let key = DataKey::Allowance(AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        });

        env.storage().persistent().set(&key, &amount);

        Approval {
            from,
            spender,
            amount: amount.try_into().unwrap(),
            live_until_ledger: live_until_ledger.into_val(&env),
        }
        .publish(&env);
    }

    pub fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();
        let sender_balance = Self::balance(env.clone(), from.clone());

        let to_address = to.address();
        let receiver_balance = Self::balance(env.clone(), to_address.clone());

        if sender_balance < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(sender_balance - amount));

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to_address.clone()), &(receiver_balance + amount));

        Transfer {
            from,
            to: to_address,
            amount: amount.try_into().unwrap(),
        }
        .publish(&env);
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) {
        spender.require_auth();

        let allowance_key = DataKey::Allowance(AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        });

        let allowed: i128 = env.storage().persistent().get(&allowance_key).unwrap_or(0);

        if allowed < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        let sender_balance = Self::balance(env.clone(), from.clone());
        let receiver_balance = Self::balance(env.clone(), to.clone());

        if sender_balance < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(sender_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(receiver_balance + amount));
        env.storage()
            .persistent()
            .set(&allowance_key, &(allowed - amount));

        Transfer {
            from,
            to,
            amount: amount.try_into().unwrap(),
        }
        .publish(&env);
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        let from_balance = Self::balance(env.clone(), from.clone());

        if from_balance < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        let allowance_key = DataKey::Allowance(AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        });

        let allowed: i128 = env.storage().persistent().get(&allowance_key).unwrap_or(0);

        if allowed < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        let from_balance = Self::balance(env.clone(), from.clone());

        if from_balance < amount {
            panic_with_error!(&env, ContractError::InsufficientFunds);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&allowance_key, &(allowed - amount));
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let current = Self::balance(env.clone(), to.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(current + amount));
    }

    pub fn decimals(_env: Env) -> u32 {
        18
    }

    pub fn name(env: Env) -> String {
        String::from_str(&env, "SibToken")
    }

    pub fn symbol(env: Env) -> String {
        String::from_str(&env, "SIB")
    }
}

impl TokenInterface for SibToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        Self::allowance(env, from, spender)
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, live_until_ledger: u32) {
        Self::approve(env, from, spender, amount, live_until_ledger)
    }

    fn balance(env: Env, id: Address) -> i128 {
        Self::balance(env, id)
    }

    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        Self::transfer(env, from, to, amount)
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        Self::transfer_from(env, spender, from, to, amount)
    }

    fn burn(env: Env, from: Address, amount: i128) {
        Self::burn(env, from, amount)
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        Self::burn_from(env, spender, from, amount)
    }

    fn decimals(env: Env) -> u32 {
        Self::decimals(env)
    }

    fn name(env: Env) -> String {
        Self::name(env)
    }

    fn symbol(env: Env) -> String {
        Self::symbol(env)
    }
}
