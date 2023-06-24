#![cfg(test)]

use crate::{ProposalContract, ProposalContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol, Vec};

#[test]
fn voters_add_and_retrieve_works() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProposalContract);
    let admin = Address::random(&env);
    let client = ProposalContractClient::new(&env, &contract_id);

    client.init(&admin);

    let voter = Address::random(&env);
    let mut voters = Vec::new(&env);
    voters.push_back(voter.clone());
    client.add_voters(&voters);

    let voters_reg = client.get_voters();

    assert_eq!(1, voters_reg.len());
    assert_eq!(voter, voters_reg.get(0).unwrap().unwrap());
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn voter_list_no_admin_cant_vote() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProposalContract);
    let client = ProposalContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);

    let mut voters = Vec::new(&env);
    voters.push_back(Address::random(&env));

    client.init(&admin);
    client.add_voters(&voters);
}

#[test]
fn ensure_admin_auth_is_checked_adding_voters() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProposalContract);
    let client = ProposalContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let voter = Address::random(&env);
    let mut voters = Vec::new(&env);
    voters.push_back(voter.clone());

    client.init(&admin);
    client.add_voters(&voters);

    assert_eq!(
        env.auths(),
        [(
            admin.clone(),
            client.address.clone(),
            Symbol::new(&env, "add_voters"),
            (voters,).into_val(&env)
        )]
    )
}
