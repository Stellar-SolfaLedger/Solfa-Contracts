#![no_std]
pub mod errors;
pub mod events;
pub mod types;

use errors::ContractError;
use events::Events;
use types::{DataKey, Plan};

use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

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

    /// Configures or updates a subscription plan.
    /// Requires admin authorization.
    pub fn set_plan(
        env: Env,
        plan_id: u32,
        duration_secs: u64,
        credits: u32,
        unlimited: bool,
        active: bool,
    ) -> Result<(), ContractError> {
        Self::require_admin(&env)?;

        if duration_secs == 0 {
            return Err(ContractError::InvalidDuration);
        }

        let plan = Plan {
            duration_secs,
            credits,
            unlimited,
            active,
        };

        env.storage().instance().set(&DataKey::Plan(plan_id), &plan);

        // Maintain list of all plan IDs
        let mut plans: Vec<u32> = env
            .storage()
            .instance()
            .get(&DataKey::PlanList)
            .unwrap_or(Vec::new(&env));

        let mut exists = false;
        for i in 0..plans.len() {
            if plans.get(i).unwrap() == plan_id {
                exists = true;
                break;
            }
        }
        if !exists {
            plans.push_back(plan_id);
            env.storage().instance().set(&DataKey::PlanList, &plans);
        }

        Events::emit_plan_updated(
            &env,
            plan_id,
            duration_secs,
            credits,
            unlimited,
            active,
        );

        Ok(())
    }

    /// Returns plan details if configured.
    pub fn get_plan(env: Env, plan_id: u32) -> Option<Plan> {
        env.storage().instance().get(&DataKey::Plan(plan_id))
    }

    /// Returns all registered plan IDs.
    pub fn get_plans(env: Env) -> Vec<u32> {
        env.storage()
            .instance()
            .get(&DataKey::PlanList)
            .unwrap_or(Vec::new(&env))
    }

    /// Sets the price for a plan in terms of a specific accepted token asset.
    /// Requires admin authorization.
    pub fn set_plan_price(
        env: Env,
        plan_id: u32,
        token: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        Self::require_admin(&env)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if Self::get_plan(env.clone(), plan_id).is_none() {
            return Err(ContractError::PlanNotFound);
        }

        env.storage()
            .instance()
            .set(&DataKey::PlanPrice(plan_id, token.clone()), &amount);

        Events::emit_plan_price_set(&env, plan_id, &token, amount);

        Ok(())
    }

    /// Returns the price for a plan in terms of a specific token.
    pub fn get_plan_price(env: Env, plan_id: u32, token: Address) -> Option<i128> {
        env.storage()
            .instance()
            .get(&DataKey::PlanPrice(plan_id, token))
    }

    /// Sets the price per transcription credit for a specific token asset.
    /// Requires admin authorization.
    pub fn set_credit_price(
        env: Env,
        token: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        Self::require_admin(&env)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        env.storage()
            .instance()
            .set(&DataKey::CreditPrice(token.clone()), &amount);

        Events::emit_credit_price_set(&env, &token, amount);

        Ok(())
    }
}
