use ic_cdk::init;
use ic_pluto::http::RawHttpRequest;
use ic_pluto::http::RawHttpResponse;
use std::cell::RefCell;

pub mod adder_mcp;
pub mod boostrap;
pub mod controller;

thread_local! {
    static API_KEY : RefCell<String> = const {RefCell::new(String::new())} ;
}

#[init]
fn init(api_key: String) {
    API_KEY.with_borrow_mut(|key| *key = api_key)
}

ic_cdk::export_candid!();
