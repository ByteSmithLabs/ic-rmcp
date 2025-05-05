mod memory;

pub trait SessionManager {
    fn create_session(&mut self) -> String;
    fn delete_session(&mut self, session_id: &str);
    fn session_exists(&self, session_id: &str) -> bool;
}