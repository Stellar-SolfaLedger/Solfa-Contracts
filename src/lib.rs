#![no_std]
pub mod errors;
pub mod events;
pub mod types;

use errors::ContractError;
use events::Events;
use types::DataKey;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct SolfaPayments;

#[contractimpl]
impl SolfaPayments {
    /// Initialises contract with admin, treasury, and operator addresses.
    /// Can only be invoked once.
    pub fn init(
        env: Env,
        admin: Address,
        treasury: Address,
        operator: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage().instance().set(&DataKey::Operator, &operator);
        env.storage().instance().set(&DataKey::Paused, &false);

        Events::emit_init(&env, &admin, &treasury, &operator);

        Ok(())
    }
}
