# SolfaLedger Payments Contract (`solfa-contracts`)

Soroban (Rust) smart contract handling subscription management and pay-per-use credits for [SolfaLedger](https://github.com/Stellar-SolfaLedger), settled on Stellar with Soroban token contracts (SACs) such as native XLM, USDC, and USDT.

---

## Contract Overview: `SolfaPayments`

`SolfaPayments` manages on-chain payments, subscription lifecycles, and backend operator entitlements for AI audio-to-solfa transcription.

### Key Features
- **Deterministic Plans**: Fixed-duration plans (e.g. 30 days) with either bundled credits or unmetered (unlimited) access.
- **Continuous Renewals**: Subscribing before expiration extends the user's active window from the existing `expires_at` timestamp.
- **Pay-Per-Use Credits**: Buy credits in bulk paying with any accepted token asset.
- **Role-Separated Operator**: Dedicated operator key invoked by backend worker to call `consume_credit` without possessing treasury or administrative keys.
- **Unlimited Pass-Through**: Users on active unlimited plans bypass credit decrement when `consume_credit` is called.
- **Emergency Circuit Breaker**: Administrative `pause` switch freezing mutating payment operations during incidents.
- **Admin Refund**: Treasury adjustment and user refund capabilities.

---

## Contract Interface & Functions

### Administrative Methods
| Function | Parameters | Description |
|---|---|---|
| `init` | `admin: Address, treasury: Address, operator: Address` | Initialises contract roles. Can only be invoked once. |
| `set_operator` | `new_operator: Address` | Updates worker operator address. Requires admin auth. |
| `set_treasury` | `new_treasury: Address` | Updates address receiving payments. Requires admin auth. |
| `set_pause` | `paused: bool` | Enables/disables emergency pause. Requires admin auth. |
| `set_plan` | `plan_id: u32, duration_secs: u64, credits: u32, unlimited: bool, active: bool` | Configures plan tier. Requires admin auth. |
| `set_plan_price` | `plan_id: u32, token: Address, amount: i128` | Sets token price (in 7-decimal stroops) for a plan. |
| `set_credit_price`| `token: Address, amount: i128` | Sets token price per single credit. |
| `refund` | `to: Address, token: Address, amount: i128` | Issues token refund from treasury to user. |

### User Payment Methods
| Function | Parameters | Description |
|---|---|---|
| `subscribe` | `user: Address, plan_id: u32, token: Address` | Transfers token payment to treasury, creates or extends subscription, and adds credits. |
| `buy_credits` | `user: Address, token: Address, count: u32` | Transfers `count * credit_price` to treasury and increments user credits. |

### Operator & Read Methods
| Function | Parameters | Return Type | Description |
|---|---|---|---|
| `consume_credit` | `user: Address, job_id: String` | `Result<(), ContractError>` | Decrements 1 credit if not unlimited. Requires operator auth. |
| `can_transcribe` | `user: Address` | `bool` | Returns `true` if user has active unlimited sub or credits > 0. |
| `get_subscription`| `user: Address` | `Option<Subscription>` | Returns active or past subscription details. |
| `get_credits` | `user: Address` | `u32` | Returns user's credit balance. |
| `get_plan` | `plan_id: u32` | `Option<Plan>` | Returns plan details. |
| `get_plans` | `()` | `Vec<u32>` | Returns all registered plan IDs for discovery. |
| `get_plan_price` | `plan_id: u32, token: Address` | `Option<i128>` | Returns price for plan in token. |
| `get_credit_price`| `token: Address` | `Option<i128>` | Returns unit credit price in token. |
| `is_paused` | `()` | `bool` | Returns true if circuit breaker is engaged. |

---

## Event Schema

All events are emitted through Soroban's event system:

| Event Topic | Topic Data | Payload Schema | Description |
|---|---|---|---|
| `init` | `admin: Address` | `(treasury: Address, operator: Address)` | Emitted on contract initialization |
| `oper_upd` | `new_operator: Address` | `()` | Emitted when operator key is updated |
| `tres_upd` | `new_treasury: Address` | `()` | Emitted when treasury address changes |
| `pause` | `()` | `paused: bool` | Emitted when contract pause state toggles |
| `plan_upd` | `plan_id: u32` | `(duration_secs, credits, unlimited, active)` | Emitted when a plan tier is modified |
| `price_set`| `plan_id: u32, token: Address` | `amount: i128` | Emitted when a plan price is set |
| `c_price` | `token: Address` | `amount: i128` | Emitted when credit price is set |
| `sub` | `user: Address, plan_id: u32` | `(token: Address, amount: i128, expires_at: u64)` | Emitted on user subscription / renewal |
| `buy_cred` | `user: Address` | `(token: Address, count: u32, amount: i128)` | Emitted when credits are purchased |
| `use_cred` | `user: Address` | `job_id: String` | Emitted when a credit is consumed |
| `refund` | `to: Address, token: Address` | `amount: i128` | Emitted on admin refund execution |

---

## Unit Testing

Run the full unit test suite covering single-init, auth restrictions, renewals, mathematical calculations, unlimited plan bypass, and circuit breakers:

```bash
cargo test
```

All 18 unit tests validate edge cases against `soroban-sdk` testutils and mock Stellar Asset Contracts.

---

## Deploying to Stellar Testnet

### Prerequisites
- Rust & `wasm32-unknown-unknown` target
- Stellar CLI (`cargo install --locked stellar-cli`)

```bash
# Automated deployment script
chmod +x scripts/deploy-testnet.sh
./scripts/deploy-testnet.sh
```

### Manual Deployment Sequence

```bash
# 1. Identities (admin/treasury, and a separate operator key used by the backend)
stellar keys generate admin    --network testnet --fund
stellar keys generate operator --network testnet --fund

# 2. Build + deploy
stellar contract build
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/solfa_payments.wasm \
  --source admin --network testnet --alias solfa

# 3. Initialise
stellar contract invoke --id solfa --source admin --network testnet -- init \
  --admin $(stellar keys address admin) \
  --treasury $(stellar keys address admin) \
  --operator $(stellar keys address operator)

# 4. Token contract IDs (Stellar Asset Contracts)
XLM=$(stellar contract id asset --asset native --network testnet)
USDC=$(stellar contract id asset --network testnet \
  --asset USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5)

# 5. Plans: id 1 = Monthly Basic (30d, 20 credits), id 2 = Pro (30d, unlimited)
stellar contract invoke --id solfa --source admin --network testnet -- \
  set_plan --plan_id 1 --duration_secs 2592000 --credits 20 --unlimited false --active true
stellar contract invoke --id solfa --source admin --network testnet -- \
  set_plan --plan_id 2 --duration_secs 2592000 --credits 0 --unlimited true --active true

# 6. Prices (7-decimal units on Stellar: 10 XLM = 100000000, 5 USDC = 50000000)
stellar contract invoke --id solfa --source admin --network testnet -- \
  set_plan_price --plan_id 1 --token $XLM  --amount 100000000
stellar contract invoke --id solfa --source admin --network testnet -- \
  set_plan_price --plan_id 1 --token $USDC --amount 50000000
stellar contract invoke --id solfa --source admin --network testnet -- \
  set_credit_price --token $XLM --amount 10000000

# 7. Smoke test
stellar contract invoke --id solfa --source admin --network testnet -- \
  subscribe --user $(stellar keys address admin) --plan_id 1 --token $XLM
stellar contract invoke --id solfa --network testnet -- \
  can_transcribe --user $(stellar keys address admin)
```

---

## Production / Mainnet Security Notes

1. **Multisig Admin**: The Admin account on Mainnet should always be configured with multi-signature authorization (e.g. 2-of-3 or 3-of-5 threshold) or a timelocked governance address.
2. **Dedicated Operator Key**: Never share or reuse the Operator key for Treasury or Administration. The backend only requires the operator key to execute `consume_credit`.
3. **No Private Keys in Repositories**: All private keys (`S...` secret keys) must be loaded from HSM, cloud secret managers (AWS Secrets Manager / Vault), or encrypted environment variables.
4. **Trustlines**: Ensure users have a trustline established to USDC or USDT prior to invoking `subscribe` or `buy_credits` with those assets.
5. **Persistent Storage & TTL Archival**: User subscriptions and credit balances are stored in `env.storage().persistent()` to prevent unbounded growth of `instance()` storage (which has a strict 64 KB limit). Each read/write automatically bumps entry TTL by 518,400 ledgers (~30 days).
6. **Refund Dual Authorization**: `refund(to, token, amount)` enforces dual-authorization requiring both `admin.require_auth()` and `treasury.require_auth()` to ensure funds cannot be unilaterally withdrawn without treasury consent.
7. **Contract Upgradability**: Code upgrades can be executed via `upgrade(new_wasm_hash)` with Admin authorization, maintaining all persistent user credit and subscription state.
