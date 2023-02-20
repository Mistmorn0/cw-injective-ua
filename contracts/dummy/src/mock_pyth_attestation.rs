use std::marker::PhantomData;

use cosmwasm_std::{
    Api, CustomQuery, DepsMut, Env, MessageInfo, OwnedDeps, Querier, QuerierWrapper, Response,
    StdResult, Storage, Uint64,
};
use schemars::_serde_json::to_string;

use injective_cosmwasm::{
    create_relay_pyth_prices_msg, Hash, InjectiveMsgWrapper, InjectiveQueryWrapper,
    PriceAttestation, PythStatus,
};

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::ContractError;

pub fn execute_trigger_pyth_update(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    price: u32,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    deps.api.debug("Starting trigger update");
    let mut response = Response::new();
    let pa = PriceAttestation {
        product_id: "MOCK_PRODUCT_ID".to_string(),
        price_id: Hash::from_hex(
            "f9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b",
        )?,
        price: price.into(),
        conf: Uint64::new(500),
        expo: -3,
        ema_price: Uint64::new(1000),
        ema_conf: Uint64::new(2000),
        status: PythStatus::Trading,
        num_publishers: 10,
        max_num_publishers: 20,
        attestation_time: Uint64::new(env.block.time.nanos() - 100),
        publish_time: Uint64::new(env.block.time.nanos()),
    };
    deps.api.debug(&format!("Msg: {}", to_string(&pa).unwrap()));
    let relay_msg = create_relay_pyth_prices_msg(env.contract.address, vec![pa]);
    response = response.add_message(relay_msg);
    Ok(response)
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn test_send_pyth() {
        let sender_addr = "inj1x2ck0ql2ngyxqtw8jteyc0tchwnwxv7npaungt";

        let msg = ExecuteMsg::TriggerPythUpdate { price: 10000 };
        let info = mock_info(sender_addr, &[]);
        let env = mock_env();
        let res = execute(inj_mock_deps().as_mut_deps(), env, info, msg);
        assert!(res.is_ok())
    }

    use std::marker::PhantomData;
    use cosmwasm_std::{Addr, Api, BlockInfo, ContractInfo, CustomQuery, DepsMut, Env, OwnedDeps, Querier, QuerierWrapper, Storage, Timestamp, TransactionInfo};
    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
    use injective_cosmwasm::{InjectiveQueryWrapper, WasmMockQuerier};
    use crate::contract::execute;
    use crate::msg::ExecuteMsg;
    //
    // pub fn inj_mock_env() -> Env {
    //     Env {
    //         block: BlockInfo {
    //             height: 12_345,
    //             time: Timestamp::from_nanos(1_571_797_419_879_305_533),
    //             chain_id: "cosmos-testnet-14002".to_string(),
    //         },
    //         transaction: Some(TransactionInfo { index: 3 }),
    //         contract: ContractInfo {
    //             address: Addr::unchecked(TEST_CONTRACT_ADDR),
    //         },
    //     }
    // }

    pub fn inj_mock_deps() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, InjectiveQueryWrapper> {
        let mut custom_querier: WasmMockQuerier = WasmMockQuerier::new();
        OwnedDeps {
            api: MockApi::default(),
            storage: MockStorage::default(),
            querier: custom_querier,
            custom_query_type: PhantomData::default(),
        }
    }

    pub trait OwnedDepsExt<S, A, Q, C>
    where
        C: CustomQuery,
    {
        fn as_mut_deps(&mut self) -> DepsMut<C>;
    }

    impl<S, A, Q, C> OwnedDepsExt<S, A, Q, C> for OwnedDeps<S, A, Q, C>
    where
        S: Storage,
        A: Api,
        Q: Querier,
        C: CustomQuery,
    {
        fn as_mut_deps(&mut self) -> DepsMut<C> {
            return DepsMut {
                storage: &mut self.storage,
                api: &self.api,
                querier: QuerierWrapper::new(&self.querier),
            };
        }
    }
    //
    // fn mock_attestation(prod: Option<[u8; 32]>, price: Option<[u8; 32]>) -> PriceAttestation {
    //     let product_id_bytes = prod.unwrap_or([21u8; 32]);
    //     let price_id_bytes = price.unwrap_or([222u8; 32]);
    //     PriceAttestation {
    //         product_id: Identifier::new(product_id_bytes),
    //         price_id: Identifier::new(price_id_bytes),
    //         price: 0x2bad2feed7,
    //         conf: 101,
    //         ema_price: -42,
    //         ema_conf: 42,
    //         expo: -3,
    //         status: PriceStatus::Trading,
    //         num_publishers: 123212u32,
    //         max_num_publishers: 321232u32,
    //         attestation_time: (0xdeadbeeffadedeedu64) as i64,
    //         publish_time: 0xdadebeefi64,
    //         prev_publish_time: 0xdeadbabei64,
    //         prev_price: 0xdeadfacebeefi64,
    //         prev_conf: 0xbadbadbeefu64, // I could do this all day -SD
    //         last_attested_publish_time: (0xdeadbeeffadedeafu64) as i64,
    //     }
    // }
}
