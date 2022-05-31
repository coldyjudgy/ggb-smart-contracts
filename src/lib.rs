use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::collections::LookupSet;
use near_sdk::{env, ext_contract, near_bindgen, PanicOnDefault, AccountId, Promise, PromiseResult, Gas};

// token 발행 + whitelist 컨트랙트
fn get_whitelist_contract() -> AccountId {
    "may17-3.testnet".to_string() // fixed
}

// promise 성공여부 확인
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
    pub owner_id: AccountId, // 돈을 받을 계정(총대)
    pub price: u128, // 공구 가격
    pub option: LookupMap<String, String>, // 공구 옵션
    pub shipping: LookupMap<String, String>, // 배송지
    pub whitelist: LookupSet<AccountId>,
}

// GAS_FOR_CALLBACK이 GAS_FOR_TRANSFER_CALL보다 30T 이상 크지 않으면 에러
const GAS_FOR_CALLBACK: Gas = 150_000_000_000_000;
const GAS_FOR_TRANSFER_CALL: Gas = 60_000_000_000_000;

// 외부 컨트랙트 콜을 하기 위한 macro
#[ext_contract(ext_whitelist)]
pub trait ExtWhitelist {
    fn is_whitelisted(&mut self, account_id: AccountId) -> bool;
}

#[ext_contract(ext_addinfo)]
pub trait ExtAddinfo {
    fn add_info(&mut self, shipping: String, option: String) -> bool;
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn is_whitelisted_callback(&mut self, shipping: String, option: String) -> bool;
}

#[near_bindgen]
impl StatusMessage {
    #[init]
    // 공구 정보 초기화
    pub fn new(owner_id: AccountId, price: u128) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Initialization only can come from the contract owner"
        );

        Self {
            owner_id,
            price,
            option: LookupMap::new(b"r".to_vec()),
            shipping: LookupMap::new(b"r".to_vec()),
            whitelist: LookupSet::new(b"f".to_vec()),
        }
    }

    // whitelist 확인 -> 송금 -> 구매자 정보 추가
    pub fn buy(&mut self, shipping: String, option: String) -> Promise {
         // whitelist 확인
        ext_whitelist::is_whitelisted(
            env::signer_account_id(), 
            &get_whitelist_contract(), 
            0, 
            GAS_FOR_TRANSFER_CALL)
            .then(
            ext_self::is_whitelisted_callback(
                shipping,
                option,
                &env::current_account_id(),
                0,
                GAS_FOR_CALLBACK,
            ),
            )
    }

    pub fn is_whitelisted_callback(&mut self, shipping: String, option: String) -> bool {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        
        // whitelist에 존재할 때(promise 성공)만 송금 -> 구매자 정보 추가 
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                // whitelist 확인
                assert_eq!(
                    near_sdk::serde_json::from_slice::<bool>(&val).unwrap(),
                    true,
                    "Authorize your account first"
                );

                Promise::new(self.owner_id.clone()).transfer(self.price
                ).then(ext_addinfo::add_info(
                    shipping,
                    option,
                    &env::current_account_id(),
                    0,
                    GAS_FOR_TRANSFER_CALL,
                ));
                near_sdk::serde_json::from_slice::<bool>(&val).unwrap()
            },
            PromiseResult::Failed => env::panic(b"ERR_CALL_FAILED"),
        }
    }

    // 송금 확인되면 구매자 정보 추가
    pub fn add_info(&mut self, shipping: String, option: String) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "'add_info' only can come from the contract owner"
        );
        let account_id = env::signer_account_id();
        let transfer_succeeded = is_promise_success();
        if transfer_succeeded {
            self.option.insert(&account_id, &option);
            self.shipping.insert(&account_id, &shipping);
        } // TODO: else 추가
        transfer_succeeded // transfer 실패하면 false 
    }

    // 구매자 정보 조회
    pub fn get_option(&self, account_id: String) -> Option<String> {
        return self.option.get(&account_id);
    }

    // 총대만 조회 가능
    pub fn get_shipping(&self, account_id: String) -> Option<String> {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "'get_shipping' only can come from the contract owner"
        );
        return self.shipping.get(&account_id);
    }
}