use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::collections::LookupSet;
use near_sdk::{env, ext_contract, near_bindgen, PanicOnDefault, AccountId, Promise, PromiseResult, Gas};

fn get_whitelist_contract() -> AccountId {
    "may17-3.testnet".to_string() // 일단 고정
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

const GAS_FOR_CALLBACK: Gas = 150_000_000_000_000;
const GAS_FOR_TRANSFER_CALL: Gas = 60_000_000_000_000;

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
         // 화이트리스트에 있는지 확인
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

        // Promise::new(self.owner_id.clone()).transfer(self.price
        // ).then(ext_addinfo::add_info(
        //     shipping,
        //     option,
        //     &env::current_account_id(),
        //     0,
        //     GAS_FOR_TRANSFER_CALL,
        // ))
    }

    pub fn is_whitelisted_callback(&mut self, shipping: String, option: String) -> bool {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        
        // promise 결과에 따라 다르게 작용
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
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

    // 송금 확인되면 컨트랙트에 구매자 정보 추가
    pub fn add_info(&mut self, shipping: String, option: String) -> bool {
        let account_id = env::signer_account_id();
        let transfer_succeeded = is_promise_success();
        if transfer_succeeded {
            self.option.insert(&account_id, &option);
            self.shipping.insert(&account_id, &shipping); // 여기서 실패해도 false
        } // else 추가
        transfer_succeeded // transfer 실패하면 얘는 자연스럽게 false 
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