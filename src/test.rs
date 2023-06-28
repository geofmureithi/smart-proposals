#![cfg(test)]

use crate::{ProposalContract, ProposalContractClient, ProposalState, Status};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, Map, Symbol, Vec};

#[test]
fn voters_add_and_retrieve_works() {
    let (env, client, _) = prepare_env_and_client();

    env.mock_all_auths();

    let mut voters = Vec::new(&env);
    voters.push_back(Address::random(&env));
    voters.push_back(Address::random(&env));
    client.add_voters(&voters);

    let voters_reg = client.get_voters();

    assert_eq!(2, voters_reg.len());
    assert_eq!(
        voters.get(0).unwrap().unwrap(),
        voters_reg.get(0).unwrap().unwrap()
    );
}

fn prepare_env_and_client<'a>() -> (Env, ProposalContractClient<'a>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProposalContract);
    let client = ProposalContractClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    client.init(&admin);

    (env, client, admin)
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn voter_list_no_admin_cant_vote() {
    let (env, client, _) = prepare_env_and_client();

    let mut voters = Vec::new(&env);
    voters.push_back(Address::random(&env));

    client.add_voters(&voters);
}

#[test]
fn ensure_admin_auth_is_checked_adding_voters() {
    let (env, client, admin) = prepare_env_and_client();

    env.mock_all_auths();

    let voter = Address::random(&env);
    let mut voters = Vec::new(&env);
    voters.push_back(voter);

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
    let (env, client, admin) = prepare_env_and_client();

    env.mock_all_auths();

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
            voters: Map::<Address, bool>::new(&env)
        },
        state
    );
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn only_admin_can_create_prds() {
    let (_, client, _) = prepare_env_and_client();
    client.create_prd(&1);
}

#[test]
fn voter_can_vote_prds() {
    let (env, client, _) = prepare_env_and_client();
    env.mock_all_auths();

    let mut voters = Vec::new(&env);
    voters.push_back(Address::random(&env));
    voters.push_back(Address::random(&env));
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id);
    client.prd_vote(&client.address, &prd_id);

    let state = client.prd_status(&prd_id);

    let mut expected_voters = Map::<Address, bool>::new(&env);
    expected_voters.set(client.address, true);

    assert_eq!(
        ProposalState {
            status: Status::OpenVoting,
            votes: 1,
            voters: expected_voters
        },
        state
    );
}

#[test]
#[should_panic(expected = "ContractError(4)")]
fn voter_cannot_vote_a_prd_twice() {
    let (env, client, _) = prepare_env_and_client();
    env.mock_all_auths();

    let mut voters = Vec::new(&env);
    voters.push_back(Address::random(&env));
    voters.push_back(Address::random(&env));
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id);
    client.prd_vote(&client.address, &prd_id);
    client.prd_vote(&client.address, &prd_id); // Double voting here. Expected panic.
}
