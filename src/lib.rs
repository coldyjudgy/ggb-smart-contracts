use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::collections::LookupSet;
use near_sdk::{env, ext_contract, near_bindgen, PanicOnDefault, AccountId, Promise, PromiseResult, Gas};
use near_sdk::utils::promise_result_as_success;

// near_sdk::setup_alloc!();

fn get_whitelist_contract() -> AccountId {
    "may17-3.testnet".to_string() // 수정 필요
}

fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StatusMessage {
    pub owner_id: AccountId,
    pub price: u128, // 제품 가격
    pub option: LookupMap<String, String>,
    pub shipping: LookupMap<String, String>,
    pub whitelist: LookupSet<AccountId>,
}

const GAS_FOR_TRANSFER_CALL: Gas = 60_000_000_000_000;
// let mut check: bool = false;

#[ext_contract(ext_whitelist)]
pub trait ExtWhitelist {
    fn is_whitelisted(&mut self, account_id: AccountId) -> bool;
}

#[ext_contract(ext_addinfo)]
pub trait ExtAddinfo {
    fn add_info(&mut self, shipping: String, option: String) -> bool;
}

// #[ext_contract(ext_self)]
// pub trait ExtSelf {
//     fn callback_promise_result() -> bool;
//     // fn callback_arg_macro(#[callback] val: bool) -> bool;
// }

// impl Default for StatusMessage {
//     fn default() -> Self {
//         Self {
//             owner_id,
//             price,
//             option: LookupMap::new(b"r".to_vec()),
//             shipping: LookupMap::new(b"r".to_vec()),
//             whitelist: LookupSet::new(b"f".to_vec()),
//         }
//     }
// }

#[near_bindgen]
impl StatusMessage {
    /// Initialize contract with an account that minted NFT.
    #[init]
    pub fn new(owner_id: AccountId, price: u128) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id, // fixed value
            price, // fixed value
            option: LookupMap::new(b"r".to_vec()),
            shipping: LookupMap::new(b"r".to_vec()),
            whitelist: LookupSet::new(b"f".to_vec()),
        }
    }


    // 화이트리스트 확인 -> 송금 -> 컨트랙트에 구매자 정보 추가
    pub fn buy(&mut self, shipping: String, option: String) -> Promise {
        ext_whitelist::is_whitelisted(
            env::signer_account_id(), 
            &get_whitelist_contract(), 
            0, 
            GAS_FOR_TRANSFER_CALL);
            
            // .then(
            // ext_self::callback_promise_result(
            //     &env::current_account_id(),
            //     0,
            //     GAS_FOR_TRANSFER_CALL,
            // ),
            // );

        let check: bool = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                near_sdk::serde_json::from_slice::<bool>(&val).unwrap().into()
            },
            PromiseResult::Failed => env::panic(b"ERR_CALL_FAILED"),
        };

        // 화이트리스트에 있는지 확인
        assert_eq!(
            check, 
            true,
            "Authorize your account first"
        );
        
        Promise::new(self.owner_id.clone()).transfer(self.price).then(ext_addinfo::add_info(
            shipping,
            option,
            &env::current_account_id(),
            1,
            GAS_FOR_TRANSFER_CALL,
        ))
    }

    // fn callback_promise_result(&mut self) -> bool {
    //     assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
    //     // promise 결과에 따라 다르게 작용
    //     match env::promise_result(0) {
    //         PromiseResult::NotReady => unreachable!(),
    //         PromiseResult::Successful(val) => {
    //             if let Ok(is_whitelisted) = near_sdk::serde_json::from_slice::<bool>(&val) {
                    
    //                 is_whitelisted
    //             } else {
    //                 env::panic(b"ERR_WRONG_VAL_RECEIVED")
    //             }
    //         },
    //         PromiseResult::Failed => env::panic(b"ERR_CALL_FAILED"),
    //     }
    // }

    // 송금 확인되면 컨트랙트에 구매자 정보 추가
    fn add_info(&mut self, shipping: String, option: String) -> bool {
        let account_id = env::signer_account_id();
        let creation_succeeded = is_promise_success();
        if creation_succeeded {
            self.option.insert(&account_id, &option);
            self.shipping.insert(&account_id, &shipping); // 여기서 실패해도 false
        } // else 추가
        creation_succeeded // transfer 실패하면 얘는 자연스럽게 false 
    }

    pub fn get_option(&self, account_id: String) -> Option<String> {
        return self.option.get(&account_id);
    }

    // 총대(contract owner)만 shipping정보 조회가능
    pub fn get_shipping(&self, account_id: String) -> Option<String> {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Get_shipping only can come from the contract owner"
        );
        return self.shipping.get(&account_id);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    // #[test]
    // fn set_get_message() {
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let mut contract = StatusMessage::StatusMessage();
    //     contract.set_status("hello".to_string());
    //     assert_eq!(
    //         "hello".to_string(),
    //         contract.get_status("bob_near".to_string()).unwrap()
    //     );
    // }

    // #[test]
    // fn get_nonexistent_message() {
    //     let context = get_context(vec![], true);
    //     testing_env!(context);
    //     let contract = StatusMessage::default();
    //     assert_eq!(None, contract.get_status("francis.near".to_string()));
    // }
}