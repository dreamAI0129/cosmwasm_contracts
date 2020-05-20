//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{coins, from_binary, Coin, InitResponse};
use cosmwasm_vm::testing::{
    init, mock_dependencies, mock_env, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::Instance;

use maker::msg::{ConfigResponse, InitMsg, QueryMsg};
use terra_bindings::TerraMsg;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/maker.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

const DEFAULT_GAS_LIMIT: u64 = 500_000;

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    // TODO: check_wasm is not exported from cosmwasm_vm
    // let terra_features = features_from_csv("staking,terra");
    // check_wasm(wasm, &terra_features).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        ask: "BTC".into(),
        offer: "ETH".into(),
    };
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse<TerraMsg> = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("BTC", value.ask.as_str());
    assert_eq!("ETH", value.offer.as_str());
    assert_eq!("creator", value.owner.as_str());
}

// #[test]
// fn increment() {
//     let mut deps = mock_instance(WASM, &coins(2, "token"));
//
//     let msg = InitMsg { count: 17 };
//     let env = mock_env(&deps.api, "creator", &coins(2, "token"));
//     let _res: InitResponse = init(&mut deps, env, msg).unwrap();
//
//     // beneficiary can release it
//     let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
//     let msg = HandleMsg::Increment {};
//     let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();
//
//     // should increase counter by 1
//     let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
//     let value: CountResponse = from_binary(&res).unwrap();
//     assert_eq!(18, value.count);
// }
//
// #[test]
// fn reset() {
//     let mut deps = mock_instance(WASM, &coins(2, "token"));
//
//     let msg = InitMsg { count: 17 };
//     let env = mock_env(&deps.api, "creator", &coins(2, "token"));
//     let _res: InitResponse = init(&mut deps, env, msg).unwrap();
//
//     // beneficiary can release it
//     let unauth_env = mock_env(&deps.api, "anyone", &coins(2, "token"));
//     let msg = HandleMsg::Reset { count: 5 };
//     let res: HandleResult = handle(&mut deps, unauth_env, msg);
//     match res.unwrap_err() {
//         StdError::Unauthorized { .. } => {}
//         _ => panic!("Expected unauthorized"),
//     }
//
//     // only the original creator can reset the counter
//     let auth_env = mock_env(&deps.api, "creator", &coins(2, "token"));
//     let msg = HandleMsg::Reset { count: 5 };
//     let _res: HandleResponse = handle(&mut deps, auth_env, msg).unwrap();
//
//     // should now be 5
//     let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
//     let value: CountResponse = from_binary(&res).unwrap();
//     assert_eq!(5, value.count);
// }
