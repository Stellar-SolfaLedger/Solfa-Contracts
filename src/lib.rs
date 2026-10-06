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

impl SolfaPayments {
    pub(crate) fn get_admin(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }

    pub(crate) fn require_admin(env: &Env) -> Result<Address, ContractError> {
        let admin = Self::get_admin(env)?;
        admin.require_auth();
        Ok(admin)
    }

    pub(crate) fn get_operator(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Operator)
            .ok_or(ContractError::NotInitialized)
    }

    pub(crate) fn get_treasury(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Treasury)
            .ok_or(ContractError::NotInitialized)
    }

    pub(crate) fn require_not_paused(env: &Env) -> Result<(), ContractError> {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::ContractPaused);
        }
        Ok(())
    }
}

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

    /// Updates the operator address (backend transcription worker key).
    /// Requires admin authorization.
    pub fn set_operator(env: Env, new_operator: Address) -> Result<(), ContractError> {
        Self::require_admin(&env)?;
        env.storage().instance().set(&DataKey::Operator, &new_operator);
        Events::emit_operator_updated(&env, &new_operator);
        Ok(())
    }

    /// Updates the treasury address receiving fee payments.
    /// Requires admin authorization.
    pub fn set_treasury(env: Env, new_treasury: Address) -> Result<(), ContractError> {
        Self::require_admin(&env)?;
        env.storage().instance().set(&DataKey::Treasury, &new_treasury);
        Events::emit_treasury_updated(&env, &new_treasury);
        Ok(())
    }

    /// Sets contract paused state for emergency circuit breaker.
    /// Requires admin authorization.
    pub fn set_pause(env: Env, paused: bool) -> Result<(), ContractError> {
        Self::require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &paused);
        Events::emit_pause_toggled(&env, paused);
        Ok(())
    }

    /// Checks if contract operations are paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }
}
