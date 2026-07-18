use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::token::Client as TokenClient;
use creditline_contract::{CreditLineContract, CreditLineContractClient};
use liquidity_pool_contract::{LiquidityPoolContract, LiquidityPoolContractClient};
use merchant_registry_contract::{MerchantRegistryContract, MerchantRegistryContractClient};
use reputation_contract::{ReputationContract, ReputationContractClient};

pub struct TestEnv<'a> {
    pub env: Env,
    pub admin: Address,
    pub token: TokenClient<'a>,
    pub token_admin: Address,
    pub creditline: CreditLineContractClient<'a>,
    pub liquidity_pool: LiquidityPoolContractClient<'a>,
    pub merchant_registry: MerchantRegistryContractClient<'a>,
    pub reputation: ReputationContractClient<'a>,
    pub treasury: Address,
    pub merchant_fund: Address,
}

impl<'a> TestEnv<'a> {
    pub fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let treasury = Address::generate(&env);
        let merchant_fund = Address::generate(&env);
        
        let token_contract_id = env.register_stellar_asset_contract(token_admin.clone());
        let token = TokenClient::new(&env, &token_contract_id);
        
        // Register contracts
        let creditline_id = env.register_contract(None, CreditLineContract);
        let creditline = CreditLineContractClient::new(&env, &creditline_id);
        
        let lp_id = env.register_contract(None, LiquidityPoolContract);
        let liquidity_pool = LiquidityPoolContractClient::new(&env, &lp_id);
        
        let registry_id = env.register_contract(None, MerchantRegistryContract);
        let merchant_registry = MerchantRegistryContractClient::new(&env, &registry_id);
        
        let reputation_id = env.register_contract(None, ReputationContract);
        let reputation = ReputationContractClient::new(&env, &reputation_id);
        
        // Initialize contracts
        merchant_registry.initialize(&admin);
        
        liquidity_pool.initialize(
            &admin,
            &token_contract_id,
            &treasury,
            &merchant_fund,
        );
        
        creditline.initialize(
            &admin,
            &reputation_id,
            &registry_id,
            &lp_id,
            &token_contract_id,
        );
        
        // Setup reputation
        reputation.set_updater(&admin, &creditline_id, &true);
        // Note: the test will manually need to use `admin` to set updater if other addresses need to update, or we can just add `admin` as an updater for tests to be able to set initial scores.
        reputation.set_updater(&admin, &admin, &true);
        
        Self {
            env,
            admin,
            token,
            token_admin,
            creditline,
            liquidity_pool,
            merchant_registry,
            reputation,
            treasury,
            merchant_fund,
        }
    }
}
