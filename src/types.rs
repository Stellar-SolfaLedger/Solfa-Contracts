use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub duration_secs: u64,
    pub credits: u32,
    pub unlimited: bool,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub plan_id: u32,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Treasury,
    Operator,
    Paused,
    Plan(u32),
    PlanList,
    PlanPrice(u32, Address),
    CreditPrice(Address),
    Subscription(Address),
    Credits(Address),
}
