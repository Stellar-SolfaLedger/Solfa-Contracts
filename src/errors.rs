use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ContractPaused = 4,
    PlanNotFound = 5,
    PlanInactive = 6,
    PriceNotSet = 7,
    InsufficientCredits = 8,
    InvalidAmount = 9,
    InvalidDuration = 10,
    TokenTransferFailed = 11,
}
