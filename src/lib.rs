#![no_std]

use soroban_sdk::{contracterror, contractimpl, contracttype, Address, ConversionError, Env, Vec};

#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Conversion = 1,
    KeyExpected = 2,
}

impl From<ConversionError> for Error {
    fn from(_: ConversionError) -> Self {
        Error::Conversion
    }
}

#[contracttype]
#[derive(Clone, Copy)]
pub enum DataKey {
    VoterList = 0,
    Admin = 1,
}

pub struct ProposalContract;

#[contractimpl]
impl ProposalContract {
    pub fn init(env: Env, admin: Address) {
        env.storage().set(&DataKey::Admin, &admin);
        env.storage()
            .set(&DataKey::VoterList, &Vec::<Address>::new(&env))
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

        Ok(env.storage().set(&DataKey::VoterList, &voter_list))
    }

    pub fn get_voters(env: Env) -> Result<Vec<Address>, Error> {
        env.storage()
            .get(&DataKey::VoterList)
            .ok_or(Error::KeyExpected)??
    }
}

#[cfg(test)]
mod test;
