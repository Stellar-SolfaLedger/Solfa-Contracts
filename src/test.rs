#[cfg(test)]
pub mod test {
    use crate::{
        errors::ContractError,
        types::{Plan, Subscription},
        SolfaPayments, SolfaPaymentsClient,
    };
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        token, Address, Env, String,
    };

    pub struct TestFixture<'a> {
        pub env: Env,
        pub admin: Address,
        pub treasury: Address,
        pub operator: Address,
        pub user: Address,
        pub contract_id: Address,
        pub client: SolfaPaymentsClient<'a>,
        pub xlm_token: Address,
        pub xlm_client: token::Client<'a>,
        pub xlm_admin_client: token::StellarAssetClient<'a>,
        pub usdc_token: Address,
        pub usdc_client: token::Client<'a>,
        pub usdc_admin_client: token::StellarAssetClient<'a>,
    }

    impl<'a> TestFixture<'a> {
        pub fn setup() -> Self {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let treasury = Address::generate(&env);
            let operator = Address::generate(&env);
            let user = Address::generate(&env);

            let contract_id = env.register(SolfaPayments, ());
            let client = SolfaPaymentsClient::new(&env, &contract_id);

            // Register native token (XLM SAC) and custom asset (USDC SAC)
            let xlm_admin = Address::generate(&env);
            let xlm_contract = env.register_stellar_asset_contract_v2(xlm_admin.clone());
            let xlm_token = xlm_contract.address();
            let xlm_client = token::Client::new(&env, &xlm_token);
            let xlm_admin_client = token::StellarAssetClient::new(&env, &xlm_token);

            let usdc_admin = Address::generate(&env);
            let usdc_contract = env.register_stellar_asset_contract_v2(usdc_admin.clone());
            let usdc_token = usdc_contract.address();
            let usdc_client = token::Client::new(&env, &usdc_token);
            let usdc_admin_client = token::StellarAssetClient::new(&env, &usdc_token);

            // Mint initial balances
            xlm_admin_client.mint(&user, &1_000_000_000); // 100 XLM (7 decimals)
            usdc_admin_client.mint(&user, &1_000_000_000); // 100 USDC

            TestFixture {
                env,
                admin,
                treasury,
                operator,
                user,
                contract_id,
                client,
                xlm_token,
                xlm_client,
                xlm_admin_client,
                usdc_token,
                usdc_client,
                usdc_admin_client,
            }
        }
    }

    #[test]
    fn test_setup_fixture() {
        let fixture = TestFixture::setup();
        assert_eq!(fixture.xlm_client.balance(&fixture.user), 1_000_000_000);
        assert_eq!(fixture.usdc_client.balance(&fixture.user), 1_000_000_000);
    }

    #[test]
    fn test_init_success() {
        let fixture = TestFixture::setup();
        let res = fixture
            .client
            .try_init(&fixture.admin, &fixture.treasury, &fixture.operator);
        assert!(res.is_ok());
        assert_eq!(fixture.client.is_paused(), false);
    }

    #[test]
    fn test_init_cannot_reinitialize() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        let second_call = fixture
            .client
            .try_init(&fixture.admin, &fixture.treasury, &fixture.operator);
        assert_eq!(
            second_call.err(),
            Some(Ok(ContractError::AlreadyInitialized))
        );
    }

    #[test]
    fn test_admin_update_operator_and_treasury() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        let new_operator = Address::generate(&fixture.env);
        let new_treasury = Address::generate(&fixture.env);

        assert!(fixture.client.try_set_operator(&new_operator).is_ok());
        assert!(fixture.client.try_set_treasury(&new_treasury).is_ok());
    }

    #[test]
    fn test_emergency_pause_toggle() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        assert_eq!(fixture.client.is_paused(), false);
        fixture.client.set_pause(&true);
        assert_eq!(fixture.client.is_paused(), true);
        fixture.client.set_pause(&false);
        assert_eq!(fixture.client.is_paused(), false);
    }

    #[test]
    fn test_plan_crud_and_listing() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        // Create plan 1: Monthly Basic (30 days, 20 credits)
        let res = fixture
            .client
            .try_set_plan(&1, &2592000, &20, &false, &true);
        assert!(res.is_ok());

        // Create plan 2: Pro Unlimited (30 days, unlimited)
        let res2 = fixture
            .client
            .try_set_plan(&2, &2592000, &0, &true, &true);
        assert!(res2.is_ok());

        // Inspect plan 1
        let plan1 = fixture.client.get_plan(&1).unwrap();
        assert_eq!(plan1.credits, 20);
        assert_eq!(plan1.unlimited, false);
        assert_eq!(plan1.active, true);

        // Inspect plan 2
        let plan2 = fixture.client.get_plan(&2).unwrap();
        assert_eq!(plan2.unlimited, true);

        // Check get_plans helper returns both IDs
        let plans = fixture.client.get_plans();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans.get(0).unwrap(), 1);
        assert_eq!(plans.get(1).unwrap(), 2);

        // Update plan 1 to inactive
        fixture
            .client
            .set_plan(&1, &2592000, &20, &false, &false);
        assert_eq!(fixture.client.get_plan(&1).unwrap().active, false);

        // Invalid duration 0 fails
        let err = fixture
            .client
            .try_set_plan(&3, &0, &10, &false, &true);
        assert_eq!(err.err(), Some(Ok(ContractError::InvalidDuration)));
    }

    #[test]
    fn test_pricing_configuration_and_validation() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        // Setup plan 1
        fixture
            .client
            .set_plan(&1, &2592000, &20, &false, &true);

        // Set plan prices for XLM and USDC
        fixture
            .client
            .set_plan_price(&1, &fixture.xlm_token, &100_000_000);
        fixture
            .client
            .set_plan_price(&1, &fixture.usdc_token, &50_000_000);

        assert_eq!(
            fixture.client.get_plan_price(&1, &fixture.xlm_token),
            Some(100_000_000)
        );
        assert_eq!(
            fixture.client.get_plan_price(&1, &fixture.usdc_token),
            Some(50_000_000)
        );

        // Set credit price for XLM
        fixture
            .client
            .set_credit_price(&fixture.xlm_token, &10_000_000);
        assert_eq!(
            fixture.client.get_credit_price(&fixture.xlm_token),
            Some(10_000_000)
        );

        // Nonexistent plan price fails
        let err = fixture
            .client
            .try_set_plan_price(&99, &fixture.xlm_token, &10_000_000);
        assert_eq!(err.err(), Some(Ok(ContractError::PlanNotFound)));

        // Non-positive amount fails
        let err2 = fixture
            .client
            .try_set_plan_price(&1, &fixture.xlm_token, &0);
        assert_eq!(err2.err(), Some(Ok(ContractError::InvalidAmount)));
    }

    #[test]
    fn test_subscribe_with_xlm_and_credit_grant() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        fixture
            .client
            .set_plan(&1, &2592000, &20, &false, &true);
        fixture
            .client
            .set_plan_price(&1, &fixture.xlm_token, &100_000_000);

        let initial_user_bal = fixture.xlm_client.balance(&fixture.user);
        let initial_tres_bal = fixture.xlm_client.balance(&fixture.treasury);

        fixture
            .client
            .subscribe(&fixture.user, &1, &fixture.xlm_token);

        assert_eq!(
            fixture.xlm_client.balance(&fixture.user),
            initial_user_bal - 100_000_000
        );
        assert_eq!(
            fixture.xlm_client.balance(&fixture.treasury),
            initial_tres_bal + 100_000_000
        );

        // Credits allocated
        assert_eq!(fixture.client.get_credits(&fixture.user), 20);

        // Subscription record created
        let sub = fixture.client.get_subscription(&fixture.user).unwrap();
        assert_eq!(sub.plan_id, 1);
        let now = fixture.env.ledger().timestamp();
        assert_eq!(sub.expires_at, now + 2592000);
    }

    #[test]
    fn test_renewal_extends_from_current_expiry() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        fixture
            .client
            .set_plan(&1, &2592000, &20, &false, &true);
        fixture
            .client
            .set_plan_price(&1, &fixture.xlm_token, &100_000_000);

        // First subscription at t=1000
        fixture.env.ledger().set_timestamp(1000);
        fixture
            .client
            .subscribe(&fixture.user, &1, &fixture.xlm_token);
        let first_expiry = fixture
            .client
            .get_subscription(&fixture.user)
            .unwrap()
            .expires_at;
        assert_eq!(first_expiry, 1000 + 2592000);

        // Renew at t=50000 (well before first expiry)
        fixture.env.ledger().set_timestamp(50000);
        fixture
            .client
            .subscribe(&fixture.user, &1, &fixture.xlm_token);
        let second_expiry = fixture
            .client
            .get_subscription(&fixture.user)
            .unwrap()
            .expires_at;

        // Must extend from previous expiry!
        assert_eq!(second_expiry, first_expiry + 2592000);
        // Credits accumulated
        assert_eq!(fixture.client.get_credits(&fixture.user), 40);
    }

    #[test]
    fn test_expired_subscription_renewal_starts_from_now() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        fixture
            .client
            .set_plan(&1, &2592000, &20, &false, &true);
        fixture
            .client
            .set_plan_price(&1, &fixture.xlm_token, &100_000_000);

        fixture.env.ledger().set_timestamp(1000);
        fixture
            .client
            .subscribe(&fixture.user, &1, &fixture.xlm_token);

        // Advance ledger way past expiry
        fixture.env.ledger().set_timestamp(10_000_000);
        fixture
            .client
            .subscribe(&fixture.user, &1, &fixture.xlm_token);

        let sub = fixture.client.get_subscription(&fixture.user).unwrap();
        // Starts fresh from current timestamp
        assert_eq!(sub.expires_at, 10_000_000 + 2592000);
    }

    #[test]
    fn test_buy_credits_math_and_balance() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        fixture
            .client
            .set_credit_price(&fixture.xlm_token, &10_000_000);

        let user_start = fixture.xlm_client.balance(&fixture.user);
        let tres_start = fixture.xlm_client.balance(&fixture.treasury);

        // Buy 5 credits
        fixture
            .client
            .buy_credits(&fixture.user, &fixture.xlm_token, &5);

        assert_eq!(
            fixture.xlm_client.balance(&fixture.user),
            user_start - 50_000_000
        );
        assert_eq!(
            fixture.xlm_client.balance(&fixture.treasury),
            tres_start + 50_000_000
        );
        assert_eq!(fixture.client.get_credits(&fixture.user), 5);
    }

    #[test]
    fn test_unaccepted_token_rejected() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        fixture
            .client
            .set_plan(&1, &2592000, &20, &false, &true);

        // Random unpriced asset
        let unpriced_token = Address::generate(&fixture.env);

        let sub_res = fixture
            .client
            .try_subscribe(&fixture.user, &1, &unpriced_token);
        assert_eq!(sub_res.err(), Some(Ok(ContractError::PriceNotSet)));

        let buy_res = fixture
            .client
            .try_buy_credits(&fixture.user, &unpriced_token, &1);
        assert_eq!(buy_res.err(), Some(Ok(ContractError::PriceNotSet)));
    }

    #[test]
    fn test_consume_credit_success_and_decrement() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        fixture
            .client
            .set_credit_price(&fixture.xlm_token, &10_000_000);
        fixture
            .client
            .buy_credits(&fixture.user, &fixture.xlm_token, &2);
        assert_eq!(fixture.client.get_credits(&fixture.user), 2);

        let job_id = String::from_str(&fixture.env, "job-101");
        let res = fixture
            .client
            .try_consume_credit(&fixture.user, &job_id);
        assert!(res.is_ok());
        assert_eq!(fixture.client.get_credits(&fixture.user), 1);
    }

    #[test]
    fn test_consume_credit_insufficient_credits() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        let job_id = String::from_str(&fixture.env, "job-102");
        let res = fixture
            .client
            .try_consume_credit(&fixture.user, &job_id);
        assert_eq!(res.err(), Some(Ok(ContractError::InsufficientCredits)));
    }

    #[test]
    fn test_unlimited_plan_bypasses_credit_deduction() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        // Plan 2: Pro Unlimited
        fixture
            .client
            .set_plan(&2, &2592000, &0, &true, &true);
        fixture
            .client
            .set_plan_price(&2, &fixture.xlm_token, &200_000_000);

        fixture
            .client
            .subscribe(&fixture.user, &2, &fixture.xlm_token);
        assert_eq!(fixture.client.get_credits(&fixture.user), 0);

        let job_id = String::from_str(&fixture.env, "job-unlimited");
        let res = fixture
            .client
            .try_consume_credit(&fixture.user, &job_id);
        assert!(res.is_ok());
        // Credits still 0 (unmetered, no underflow)
        assert_eq!(fixture.client.get_credits(&fixture.user), 0);
    }

    #[test]
    fn test_can_transcribe_scenarios() {
        let fixture = TestFixture::setup();
        fixture
            .client
            .init(&fixture.admin, &fixture.treasury, &fixture.operator);

        // New user has no credits/sub
        assert_eq!(fixture.client.can_transcribe(&fixture.user), false);

        // User buys credits
        fixture
            .client
            .set_credit_price(&fixture.xlm_token, &10_000_000);
        fixture
            .client
            .buy_credits(&fixture.user, &fixture.xlm_token, &1);
        assert_eq!(fixture.client.can_transcribe(&fixture.user), true);

        // Consume the credit
        let job_id = String::from_str(&fixture.env, "job-1");
        fixture.client.consume_credit(&fixture.user, &job_id);
        assert_eq!(fixture.client.can_transcribe(&fixture.user), false);

        // Subscribe to unlimited plan
        fixture
            .client
            .set_plan(&2, &2592000, &0, &true, &true);
        fixture
            .client
            .set_plan_price(&2, &fixture.xlm_token, &200_000_000);
        fixture.env.ledger().set_timestamp(1000);
        fixture
            .client
            .subscribe(&fixture.user, &2, &fixture.xlm_token);
        assert_eq!(fixture.client.can_transcribe(&fixture.user), true);

        // Fast forward ledger past expiry
        fixture.env.ledger().set_timestamp(10_000_000);
        assert_eq!(fixture.client.can_transcribe(&fixture.user), false);
    }
}
