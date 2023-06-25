#![cfg(test)]

use crate::{ProposalContract, ProposalContractClient, ProposalState, Status};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, Symbol, Vec};

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
    voters.push_back(voter);

    client.init(&admin);
    client.add_voters(&voters);

    assert_eq!(
        env.auths(),
        [(
            admin,
            client.address,
            Symbol::new(&env, "add_voters"),
            (voters,).into_val(&env)
        )]
    )
}

#[test]
fn prd_creation_and_query() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProposalContract);
    let client = ProposalContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    client.init(&admin);

    let voter_1 = Address::random(&env);
    let voter_2 = Address::random(&env);
    let voters = vec![&env, voter_1, voter_2];
    client.add_voters(&voters);

    let prd_id = 1001u64;
    client.create_prd(&prd_id);

    assert_eq!(
        env.auths(),
        [(
            admin,
            client.address.clone(),
            Symbol::new(&env, "create_prd"),
            (1001u64,).into_val(&env)
        )]
    );

    let state = client.prd_status(&prd_id);

    assert_eq!(
        ProposalState {
            status: Status::OpenVoting,
            votes: 0,
        },
        state
    );
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn only_admin_can_create_prds() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProposalContract);
    let client = ProposalContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    client.init(&admin);
    
    client.create_prd(&1);
}