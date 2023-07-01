#![cfg(test)]

use crate::{Error, Proposal, ProposalContract, ProposalContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, IntoVal, Map, Symbol,
};

#[test]
fn voters_add_and_retrieve_works() {
    let (env, client, _) = setup_test();

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

fn setup_test<'a>() -> (Env, ProposalContractClient<'a>, Address) {
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
    let (env, client, _) = setup_test();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);
    client.add_voters(&voters);
}

#[test]
fn admin_auth_is_checked_adding_voters() {
    let (env, client, admin) = setup_test();

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
    let (env, client, admin) = setup_test();

    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);
    voters.set(Address::random(&env), 1);

    client.add_voters(&voters);

    let id = 1001u64;
    client.create_prd(&id, &3600);

    assert_eq!(
        env.auths(),
        [(
            admin,
            client.address.clone(),
            Symbol::new(&env, "create_prd"),
            (1001u64, 3600u64).into_val(&env)
        )]
    );

    let state = client.proposal(&id);

    assert_eq!(
        Proposal {
            id,
            kind: crate::ProposalKind::PRD,
            voting_end_time: env.ledger().timestamp() + 3600,
            parent: 0,
            votes: 0,
            voters: Map::<Address, bool>::new(&env)
        },
        state
    );
}

#[test]
#[should_panic(expected = "ContractError(7)")]
fn cannot_create_same_id_proposals() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let id = 1001u64;
    client.create_prd(&id, &3600);
    client.create_prd(&id, &3600);
}

#[test]
#[should_panic(expected = "NotAuthorized")]
fn only_admin_can_create_proposals() {
    let (_, client, _) = setup_test();
    client.create_prd(&1, &3600);
}

#[test]
fn voter_can_vote_proposals() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 2);
    client.add_voters(&voters);

    let id = 12;

    client.create_prd(&id, &3600);
    client.vote(&client.address, &id, &2);

    let state = client.proposal(&id);

    let mut expected_voters = Map::<Address, bool>::new(&env);
    expected_voters.set(client.address, true);

    assert_eq!(
        Proposal {
            id,
            kind: crate::ProposalKind::PRD,
            voting_end_time: env.ledger().timestamp() + 3600,
            parent: 0,
            votes: 2,
            voters: expected_voters
        },
        state
    );
}

#[test]
#[should_panic(expected = "ContractError(4)")]
fn voter_cannot_vote_a_proposal_twice() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 1);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id, &3600);
    client.vote(&client.address, &prd_id, &1);
    client.vote(&client.address, &prd_id, &1); // Double voting here. Expected panic.
}

#[test]
#[should_panic(expected = "ContractError(5)")]
fn not_in_voter_list_address_cant_vote() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(Address::random(&env), 1);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id, &3600);
    client.vote(&client.address, &prd_id, &1);
}

#[test]
#[should_panic(expected = "ContractError(6)")]
fn voter_cannot_vote_more_than_its_total_weight_upper_bound() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 2);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id, &3600);
    client.vote(&client.address, &prd_id, &3); // Exceeding weight of 2 should panic.
}

#[test]
#[should_panic(expected = "ContractError(6)")]
fn voter_cannot_vote_more_than_its_total_weight_lower_bound() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let mut voters = Map::<Address, u32>::new(&env);
    voters.set(client.address.clone(), 2);
    client.add_voters(&voters);

    let prd_id = 12;

    client.create_prd(&prd_id, &3600);
    client.vote(&client.address, &prd_id, &-3); // Exceeding weight of 2 should panic.
}

#[test]
fn rfc_proposal_creation() {
    let (env, client, admin) = setup_test();
    env.mock_all_auths();

    let prd_id = 1001u64;
    client.create_prd(&prd_id, &3600); // First we need a PRD.

    let mut capture_auths = env.auths();
    let rfc_id = 1002u64;

    client.create_rfc(&prd_id, &rfc_id, &3600);
    capture_auths.append(&mut env.auths());

    let state = client.proposal(&rfc_id);
    assert_eq!(
        capture_auths,
        [
            (
                admin.clone(),
                client.address.clone(),
                Symbol::new(&env, "create_prd"),
                (1001u64, 3600u64).into_val(&env)
            ),
            (
                admin.clone(),
                client.address.clone(),
                Symbol::new(&env, "create_rfc"),
                (1001u64, 1002u64, 3600u64).into_val(&env)
            )
        ]
    );

    assert_eq!(
        Proposal {
            id: rfc_id,
            kind: crate::ProposalKind::RFC,
            voting_end_time: env.ledger().timestamp() + 3600,
            parent: prd_id,
            votes: 0,
            voters: Map::<Address, bool>::new(&env)
        },
        state
    );
}

#[test]
#[should_panic(expected = "ContractError(3)")]
fn cannot_create_an_rfc_with_non_existing_parent_prd() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    let id = 1001u64;
    client.create_rfc(&id, &1, &3600);
}

#[test]
#[should_panic(expected = "ContractError(8)")]
fn no_prd_proposals_cannot_be_parent() {
    let (env, client, _) = setup_test();
    env.mock_all_auths();

    client.create_prd(&1, &3600);
    client.create_rfc(&1, &2, &3600);
    client.create_rfc(&2, &3, &3600); // Here we passed the id of an RFC, not a PRD. This should panic.
}

#[test]
fn cannot_vote_if_voting_time_exceeded() {
    let (mut env, _, _) = setup_test();

    let mut proposal = Proposal {
        id: 1,
        kind: crate::ProposalKind::PRD,
        voting_end_time: env.ledger().timestamp() + 3600,
        parent: 0,
        votes: 0,
        voters: Map::<Address, bool>::new(&env),
    };

    advance_ledger_time_in(3600, &mut env);

    let result = proposal.vote(env.ledger().timestamp(), Address::random(&env), 1);

    assert_eq!(Err(Error::VotingClosed), result)
}

fn advance_ledger_time_in(time: u64, env: &mut Env) {
    let mut ledger_info = env.ledger().get();
    ledger_info.timestamp = ledger_info.timestamp + time;
    env.ledger().set(ledger_info)
}
