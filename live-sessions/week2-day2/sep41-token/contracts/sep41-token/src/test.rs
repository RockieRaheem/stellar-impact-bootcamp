#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, MuxedAddress, String};

use crate::our_token::{SibToken, SibTokenClient};
struct SetUpResult<'a> {
    env: Env,
    client: SibTokenClient<'a>,
    admin: Address,
    sender: Address,
    receiver: Address,
}

fn setup<'a>() -> SetUpResult<'a> {
    let env = Env::default();

    env.mock_all_auths();

    let admin = Address::generate(&env);

    let initial_supply = 1_000_000i128;

    let contract_id = env.register(SibToken, (admin.clone(), initial_supply));

    let client = SibTokenClient::new(&env, &contract_id);

    let sender = Address::generate(&env);

    let receiver = Address::generate(&env);

    SetUpResult {
        env,
        client,
        admin,
        sender,
        receiver,
    }
}

#[test]
fn test_name() {
    let setup_result = setup();

    let name = setup_result.client.name();
    let token_name = String::from_str(&setup_result.env, "SibToken");
    assert_eq!(name, token_name);
}

#[test]
fn test_symbol() {
    let setup_result = setup();

    let name = setup_result.client.symbol();
    let token_name = String::from_str(&setup_result.env, "SIB");

    let not_token_name = String::from_str(&setup_result.env, "Sib");
    assert_eq!(name, token_name);
    assert_ne!(name, not_token_name);
}

#[test]
fn test_decimal() {
    let setup_result = setup();

    let decimal = setup_result.client.decimals();
    let token_decimal = 18;

    assert_eq!(decimal, token_decimal);
}

#[test]
fn test_transfer() {
    let setup_result = setup();

    let amount = 500i128;

    setup_result
        .client
        .transfer(&setup_result.admin, &MuxedAddress::Account(setup_result.receiver.clone()), &amount);

    let sender_balance = setup_result.client.balance(&setup_result.admin);
    let receiver_balance = setup_result.client.balance(&setup_result.receiver);

    assert_eq!(sender_balance, 1_000_000 - amount);
    assert_eq!(receiver_balance, amount);
}

#[test]
fn test_transfer_from() {
    let setup_result = setup();

    let spender = Address::generate(&setup_result.env);
    let amount = 300i128;

    setup_result
        .client
        .approve(&setup_result.admin, &spender, &amount, &100);

    setup_result
        .client
        .transfer_from(&spender, &setup_result.admin, &setup_result.receiver, &amount);

    let sender_balance = setup_result.client.balance(&setup_result.admin);
    let receiver_balance = setup_result.client.balance(&setup_result.receiver);
    let remaining = setup_result
        .client
        .allowance(&setup_result.admin, &spender);

    assert_eq!(sender_balance, 1_000_000 - amount);
    assert_eq!(receiver_balance, amount);
    assert_eq!(remaining, 0);
}

#[test]
fn test_burn() {
    let setup_result = setup();

    let amount = 200i128;

    setup_result
        .client
        .burn(&setup_result.admin, &amount);

    let balance = setup_result.client.balance(&setup_result.admin);
    assert_eq!(balance, 1_000_000 - amount);
}

#[test]
fn test_burn_from() {
    let setup_result = setup();

    let spender = Address::generate(&setup_result.env);
    let amount = 150i128;

    setup_result
        .client
        .approve(&setup_result.admin, &spender, &amount, &100);

    setup_result
        .client
        .burn_from(&spender, &setup_result.admin, &amount);

    let balance = setup_result.client.balance(&setup_result.admin);
    let remaining = setup_result
        .client
        .allowance(&setup_result.admin, &spender);

    assert_eq!(balance, 1_000_000 - amount);
    assert_eq!(remaining, 0);
}

#[test]
fn test_mint() {
    let setup_result = setup();

    let amount = 400i128;

    setup_result
        .client
        .mint(&setup_result.receiver, &amount);

    let receiver_balance = setup_result.client.balance(&setup_result.receiver);
    assert_eq!(receiver_balance, amount);
}
