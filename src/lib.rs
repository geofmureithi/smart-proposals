#![no_std]

use soroban_sdk::{
    contracterror, contractimpl, contracttype, Address, ConversionError, Env, Map, Vec,
};

#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Conversion = 1,
    KeyExpected = 2,
    NotFound = 3,
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
    PRDStorage = 2,
}

pub struct ProposalContract;

#[contractimpl]
impl ProposalContract {
    pub fn init(env: Env, admin: Address) {
        env.storage().set(&DataKey::Admin, &admin);
        env.storage()
            .set(&DataKey::VoterList, &Vec::<Address>::new(&env));
        env.storage().set(
            &DataKey::PRDStorage,
            &Map::<u64, (Status, i64, Vec<Address>)>::new(&env),
        )
    }

    pub fn add_voters(env: Env, voters: Vec<Address>) -> Result<(), Error> {
        env.storage()
            .get::<_, Address>(&DataKey::Admin)
            .ok_or(Error::KeyExpected)??
            .require_auth();

        let mut voter_list = env
            .storage()
            .get::<_, Vec<Address>>(&DataKey::VoterList)
            .ok_or(Error::KeyExpected)??;

        voter_list.append(&voters);

        env.storage().set(&DataKey::VoterList, &voter_list);
        Ok(())
    }

    pub fn get_voters(env: Env) -> Result<Vec<Address>, Error> {
        env.storage()
            .get(&DataKey::VoterList)
            .ok_or(Error::KeyExpected)??
    }

    pub fn create_prd(env: Env, id: u64) -> Result<(), Error> {
        env.storage()
            .get::<_, Address>(&DataKey::Admin)
            .ok_or(Error::KeyExpected)??
            .require_auth();

        let mut storage = env
            .storage()
            .get::<_, Map<u64, ProposalState>>(&DataKey::PRDStorage)
            .ok_or(Error::KeyExpected)??;

        storage.set(
            id,
            ProposalState {
                status: Status::OpenVoting,
                votes: 0,
            },
        );

        env.storage().set(&DataKey::PRDStorage, &storage);
        Ok(())
    }

    pub fn prd_status(env: Env, id: u64) -> Result<ProposalState, Error> {
        Ok(env
            .storage()
            .get::<_, Map<u64, ProposalState>>(&DataKey::PRDStorage)
            .ok_or(Error::KeyExpected)??
            .get(id)
            .ok_or(Error::NotFound)??)
    }
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProposalState {
    status: Status,
    votes: i64,
}

#[cfg(test)]
mod test;
