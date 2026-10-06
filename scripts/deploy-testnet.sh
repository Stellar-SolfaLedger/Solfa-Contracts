#!/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo "SolfaLedger: Deploy Payments Contract to Stellar Testnet"
echo "=========================================================="

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"

# 1. Identities (admin/treasury, and a separate operator key used by the backend)
echo "[1/7] Ensuring Stellar CLI identities..."
stellar keys generate admin    --network "$NETWORK" --fund || true
stellar keys generate operator --network "$NETWORK" --fund || true

ADMIN_ADDR=$(stellar keys address admin)
OPERATOR_ADDR=$(stellar keys address operator)

echo "Admin address:    $ADMIN_ADDR"
echo "Operator address: $OPERATOR_ADDR"

# 2. Build WASM contract
echo "[2/7] Building contract WASM..."
stellar contract build

WASM_PATH="target/wasm32-unknown-unknown/release/solfa_payments.wasm"
if [ ! -f "$WASM_PATH" ]; then
    WASM_PATH="target/wasm32v1-none/release/solfa_payments.wasm"
fi

# 3. Deploy contract
echo "[3/7] Deploying contract to testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source admin \
  --network "$NETWORK" \
  --alias solfa)

echo "Contract ID: $CONTRACT_ID"

# 4. Initialize contract
echo "[4/7] Initializing SolfaPayments..."
stellar contract invoke \
  --id solfa \
  --source admin \
  --network "$NETWORK" \
  -- init \
  --admin "$ADMIN_ADDR" \
  --treasury "$ADMIN_ADDR" \
  --operator "$OPERATOR_ADDR"

# 5. Token contract IDs (Stellar Asset Contracts)
echo "[5/7] Resolving token asset contract IDs..."
XLM=$(stellar contract id asset --asset native --network "$NETWORK")
USDC=$(stellar contract id asset --network "$NETWORK" \
  --asset "USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5")

echo "Native XLM Asset Contract: $XLM"
echo "Circle USDC Testnet SAC:   $USDC"

# 6. Seed Plans and Prices
echo "[6/7] Seeding subscription plans and token prices..."
# Plan 1: Monthly Basic (30 days = 2592000s, 20 credits, unlimited = false, active = true)
stellar contract invoke --id solfa --source admin --network "$NETWORK" -- \
  set_plan --plan_id 1 --duration_secs 2592000 --credits 20 --unlimited false --active true

# Plan 2: Pro Unlimited (30 days = 2592000s, 0 credits, unlimited = true, active = true)
stellar contract invoke --id solfa --source admin --network "$NETWORK" -- \
  set_plan --plan_id 2 --duration_secs 2592000 --credits 0 --unlimited true --active true

# Plan 1 Prices: 10 XLM (100,000,000 stroops), 5 USDC (50,000,000 units)
stellar contract invoke --id solfa --source admin --network "$NETWORK" -- \
  set_plan_price --plan_id 1 --token "$XLM" --amount 100000000

stellar contract invoke --id solfa --source admin --network "$NETWORK" -- \
  set_plan_price --plan_id 1 --token "$USDC" --amount 50000000

# Credit Price: 1 XLM (10,000,000 stroops) per credit
stellar contract invoke --id solfa --source admin --network "$NETWORK" -- \
  set_credit_price --token "$XLM" --amount 10000000

# 7. Smoke Test
echo "[7/7] Running smoke tests..."
stellar contract invoke --id solfa --source admin --network "$NETWORK" -- \
  subscribe --user "$ADMIN_ADDR" --plan_id 1 --token "$XLM"

CAN_TRANSCRIBE=$(stellar contract invoke --id solfa --network "$NETWORK" -- \
  can_transcribe --user "$ADMIN_ADDR")

echo "can_transcribe status for admin: $CAN_TRANSCRIBE"

# Save deployment information
mkdir -p deployments
cat <<EOF > deployments/testnet.json
{
  "network": "testnet",
  "contractId": "$CONTRACT_ID",
  "admin": "$ADMIN_ADDR",
  "operator": "$OPERATOR_ADDR",
  "treasury": "$ADMIN_ADDR",
  "tokens": {
    "XLM": "$XLM",
    "USDC": "$USDC"
  },
  "plans": [
    {
      "id": 1,
      "name": "Monthly Basic",
      "duration_secs": 2592000,
      "credits": 20,
      "unlimited": false,
      "prices": {
        "XLM": "100000000",
        "USDC": "50000000"
      }
    },
    {
      "id": 2,
      "name": "Pro Unlimited",
      "duration_secs": 2592000,
      "credits": 0,
      "unlimited": true,
      "prices": {
        "XLM": "250000000",
        "USDC": "120000000"
      }
    }
  ],
  "creditPrice": {
    "XLM": "10000000"
  }
}
EOF

echo "Deployment completed successfully. Saved to deployments/testnet.json"
