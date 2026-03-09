#[macro_export]
macro_rules! init_contract_with_admin {
    ($contract_type:path, $client_type:path, $mock_auth:expr) => {{
        let (init_data, env) = oracle::testutils::generate_test_env();

        oracle::testutils::set_ledger_timestamp(&env, 900);

        let contract_id = env.register($contract_type, ());
        let client = <$client_type>::new(&env, &contract_id);

        client.mock_all_auths().config(&init_data);

        if $mock_auth {
            env.mock_all_auths();
        }

        (env, client, init_data)
    }};
}
