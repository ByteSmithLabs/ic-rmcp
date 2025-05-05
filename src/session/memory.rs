use std::cell::RefCell;
use std::collections::HashSet;
use candid::{CandidType, Deserialize};
use super::SessionManager;

#[derive(Default, Deserialize, CandidType)]
struct WasmSessionStore {
    active_sessions: HashSet<String>,
}

impl SessionManager for WasmSessionStore {
    fn create_session(&mut self) -> String {
        let session_id = loop {
            let new_id = generate_random_session_id(16);
            if !self.active_sessions.contains(&new_id) {
                break new_id;
            }
        };
        self.active_sessions.insert(session_id.clone());
        session_id
    }

    fn delete_session(&mut self, session_id: &str) {
        self.active_sessions.remove(session_id);
    }

    fn session_exists(&self, session_id: &str) -> bool {
        self.active_sessions.contains(session_id)
    }
}

thread_local! {
    static SESSION_STORE: RefCell<WasmSessionStore> = RefCell::new(WasmSessionStore::default());
}

fn generate_random_session_id(num_bytes: usize) -> String {
    use rand::RngCore;
    let mut bytes = vec![0u8; num_bytes];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn create_new_session() -> String {
    SESSION_STORE.with(|store_refcell| {
        store_refcell.borrow_mut().create_session()
    })
}

fn end_session(session_id: String) {
    SESSION_STORE.with(|store_refcell| {
        store_refcell.borrow_mut().delete_session(&session_id);
    })
}

fn check_session(session_id: String) -> bool {
    SESSION_STORE.with(|store_refcell| {
        store_refcell.borrow().session_exists(&session_id)
    })
}