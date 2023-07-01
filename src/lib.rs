#![no_std]

use soroban_sdk::{contracterror, contractimpl, contracttype, Address, ConversionError, Env, Map};

#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Conversion = 1,
    KeyExpected = 2,
    NotFound = 3,
    AlreadyVoted = 4,
    NotInVoterList = 5,
    WeightExceeded = 6,
    DuplicatedEntity = 7,
    ParentNotPRD = 8,
}

impl From<ConversionError> for Error {
    fn from(_: ConversionError) -> Self {
        Error::Conversion
    }
}

#[contracttype]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Status {
    OpenVoting = 1,
    Approved = 2,
    Rejected = 3,
}

#[contracttype]
#[derive(Clone, Copy)]
pub enum DataKey {
    VoterList = 0,
    Admin = 1,
    ProposalStorage = 2,
}

pub struct ProposalContract;

#[contractimpl]
impl ProposalContract {
    pub fn init(env: Env, admin: Address) {
        env.storage().set(&DataKey::Admin, &admin);
        env.storage()
            .set(&DataKey::VoterList, &Map::<Address, u32>::new(&env));
        env.storage().set(
            &DataKey::ProposalStorage,
            &Map::<u64, (Status, i64, Map<Address, bool>)>::new(&env),
        )
    }

    pub fn add_voters(env: Env, voters: Map<Address, u32>) -> Result<(), Error> {
        env.storage()
            .get::<_, Address>(&DataKey::Admin)
            .ok_or(Error::KeyExpected)??
            .require_auth();

        let mut voter_list = env
            .storage()
            .get::<_, Map<Address, u32>>(&DataKey::VoterList)
            .ok_or(Error::KeyExpected)??;

        for res in voters.iter() {
            let (addr, weight) = res?;
            voter_list.set(addr, weight);
        }

        env.storage().set(&DataKey::VoterList, &voter_list);
        Ok(())
    }

    pub fn get_voters(env: Env) -> Result<Map<Address, u32>, Error> {
        env.storage()
            .get(&DataKey::VoterList)
            .ok_or(Error::KeyExpected)??
    }

    pub fn create_prd(env: Env, id: u64) -> Result<(), Error> {
        Self::create_proposal(env, id, ProposalKind::PRD, 0)
    }

    pub fn create_rfc(env: Env, prd_id: u64, id: u64) -> Result<(), Error> {
        Self::create_proposal(env, id, ProposalKind::RFC, prd_id)
    }

    fn create_proposal(env: Env, id: u64, kind: ProposalKind, parent: u64) -> Result<(), Error> {
        env.storage()
            .get::<_, Address>(&DataKey::Admin)
            .ok_or(Error::KeyExpected)??
            .require_auth();

        let mut storage = env
            .storage()
            .get::<_, Map<u64, Proposal>>(&DataKey::ProposalStorage)
            .ok_or(Error::KeyExpected)??;

        if storage.contains_key(id) {
            return Err(Error::DuplicatedEntity);
        }

        if parent != 0 {
            let parent = storage.get(parent).ok_or(Error::NotFound)??;
            if parent.kind != ProposalKind::PRD {
                return Err(Error::ParentNotPRD);
            }
        }

        storage.set(
            id,
            Proposal {
                id,
                kind,
                parent,
                status: Status::OpenVoting,
                votes: 0,
                voters: Map::<Address, bool>::new(&env),
            },
        );

        env.storage().set(&DataKey::ProposalStorage, &storage);
        Ok(())
    }

    pub fn proposal(env: Env, id: u64) -> Result<Proposal, Error> {
        Ok(env
            .storage()
            .get::<_, Map<u64, Proposal>>(&DataKey::ProposalStorage)
            .ok_or(Error::KeyExpected)??
            .get(id)
            .ok_or(Error::NotFound)??)
    }

    pub fn vote(env: Env, voter: Address, id: u64, weight: i32) -> Result<(), Error> {
        voter.require_auth();

        let voter_list = env
            .storage()
            .get::<_, Map<Address, u32>>(&DataKey::VoterList)
            .ok_or(Error::KeyExpected)??;

        let voter_weight = voter_list
            .get(voter.clone())
            .ok_or(Error::NotInVoterList)??;

        if weight.unsigned_abs() > voter_weight {
            return Err(Error::WeightExceeded);
        }

        let mut proposal_storage = env
            .storage()
            .get::<_, Map<u64, Proposal>>(&DataKey::ProposalStorage)
            .ok_or(Error::KeyExpected)??;

        let mut proposal = proposal_storage.get(id).ok_or(Error::NotFound)??;

        if proposal.voters.get(voter.clone()).is_some() {
            return Err(Error::AlreadyVoted);
        }

        proposal.votes = proposal
            .votes
            .checked_add(weight as i64)
            .expect("overflow");

        proposal.voters.set(voter, true);
        
        proposal_storage.set(id, proposal);

        env.storage()
            .set(&DataKey::ProposalStorage, &proposal_storage);
        Ok(())
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    id: u64,
    kind: ProposalKind,
    // Parent "0" means no parent.
    parent: u64,
    status: Status,
    votes: i64,
    voters: Map<Address, bool>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ProposalKind {
    PRD,
    RFC,
}

#[cfg(test)]
mod test;
