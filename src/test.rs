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
}
