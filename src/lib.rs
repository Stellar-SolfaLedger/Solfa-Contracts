#![no_std]
pub mod errors;
pub mod events;
pub mod types;

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SolfaPayments;

#[contractimpl]
impl SolfaPayments {
    pub fn hello(_env: Env) -> u32 {
        42
    }
}
