#![cfg(test)]

use crate::{Proposal, ProposalContract, ProposalContractClient, Status};
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Map, Symbol};

#[test]
fn voters_add_and_retrieve_works() {
    let (env, client, _) = prepare_env_and_client();

    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    let voter_1 = Address::random(&env);
    let voter_2 = Address::random(&env);
    voters.set(voter_1.clone(), 1);
    voters.set(voter_2, 1);
    client.add_voters(&voters);

    let voters_reg = client.get_voters();

    assert_eq!(2, voters_reg.len());
    assert_eq!(
        voters.get(voter_1.clone()).unwrap().unwrap(),
        voters_reg.get(voter_1.clone()).unwrap().unwrap()
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

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);
    client.add_voters(&voters);
}

#[test]
fn admin_auth_is_checked_adding_voters() {
    let (env, client, admin) = prepare_env_and_client();

    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);

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
fn proposal_creation_and_query() {
    let (env, client, admin) = prepare_env_and_client();

    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);
    voters.set(Address::random(&env), 1);

    client.add_voters(&voters);

    let id = 1001u64;
    client.create_prd(&id);

    assert_eq!(
        env.auths(),
        [(
            admin,
            client.address.clone(),
            Symbol::new(&env, "create_prd"),
            (1001u64,).into_val(&env)
        )]
    );

    let state = client.proposal(&id);

    assert_eq!(
        Proposal {
            id,
            kind: crate::ProposalKind::PRD,
            parent: 0,
            status: Status::OpenVoting,
            votes: 0,
            voters: Map::<Address, bool>::new(&env)
        },
        state
    );
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn only_admin_can_create_proposals() {
    let (_, client, _) = prepare_env_and_client();
    client.create_prd(&1);
}

#[test]
fn voter_can_vote_proposals() {
    let (env, client, _) = prepare_env_and_client();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 2);
    client.add_voters(&voters);

    let id = 12;

    client.create_prd(&id);
    client.vote(&client.address, &id, &2);

    let state = client.proposal(&id);

    let mut expected_voters = Map::<Address, bool>::new(&env);
    expected_voters.set(client.address, true);

    assert_eq!(
        Proposal {
            id,
            kind: crate::ProposalKind::PRD,
            parent: 0,
            status: Status::OpenVoting,
            votes: 2,
            voters: expected_voters
        },
        state
    );
}

#[test]
#[should_panic(expected = "ContractError(4)")]
fn voter_cannot_vote_a_proposal_twice() {
    let (env, client, _) = prepare_env_and_client();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 1);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id);
    client.vote(&client.address, &prd_id, &1);
    client.vote(&client.address, &prd_id, &1); // Double voting here. Expected panic.
}

#[test]
#[should_panic(expected = "ContractError(5)")]
fn not_in_voter_list_address_cant_vote() {
    let (env, client, _) = prepare_env_and_client();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id);
    client.vote(&client.address, &prd_id, &1);
}

#[test]
#[should_panic(expected = "ContractError(6)")]
fn voter_cannot_vote_more_than_its_total_weight() {
    let (env, client, _) = prepare_env_and_client();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 2);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id);
    client.vote(&client.address, &prd_id, &3); // Exceeding weight of 2 should panic.
}
