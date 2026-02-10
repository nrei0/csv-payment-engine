use super::id::ClientId;

#[derive(Debug, Clone)]
pub struct Account {
    #[allow(dead_code)] // Normally should be used, but not in our example: useful for debugging.
    pub client: ClientId,
    pub available: i64,
    pub held: i64,
    pub locked: bool,
}

impl Account {
    pub fn total(&self) -> i64 {
        self.available + self.held
    }
}
