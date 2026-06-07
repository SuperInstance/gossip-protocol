//! Rumor mongering with TTL and fanout.

pub type RumorId = u64;

/// A piece of gossip (rumor) to be spread.
#[derive(Debug, Clone)]
pub struct Rumor {
    pub id: RumorId,
    pub payload: Vec<u8>,
    pub origin: u64,
    pub ttl: u32,
    pub round_created: u64,
}

impl Rumor {
    pub fn new(id: RumorId, payload: Vec<u8>, origin: u64, round_created: u64) -> Self {
        Self { id, payload, origin, ttl: 10, round_created }
    }

    /// Decrement TTL, return true if still alive.
    pub fn tick(&mut self) -> bool {
        if self.ttl == 0 {
            return false;
        }
        self.ttl -= 1;
        self.ttl > 0
    }
}

/// Per-node rumor store.
#[derive(Debug, Clone)]
pub struct RumorStore {
    pub known_rumors: Vec<RumorId>,
    pub pending: Vec<Rumor>,
}

impl Default for RumorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RumorStore {
    pub fn new() -> Self {
        Self { known_rumors: Vec::new(), pending: Vec::new() }
    }

    /// Add a new rumor.
    pub fn add(&mut self, rumor: Rumor) -> bool {
        if self.known_rumors.contains(&rumor.id) {
            return false;
        }
        self.known_rumors.push(rumor.id);
        self.pending.push(rumor);
        true
    }

    /// Get pending rumors (not yet fully spread).
    pub fn get_pending(&self) -> &[Rumor] {
        &self.pending
    }

    /// Tick all pending rumors, removing expired ones.
    pub fn tick(&mut self) {
        for r in &mut self.pending {
            r.tick();
        }
        self.pending.retain(|r| r.ttl > 0);
    }

    /// Mark a rumor as delivered (remove from pending).
    pub fn mark_delivered(&mut self, id: RumorId) {
        self.pending.retain(|r| r.id != id);
    }

    /// Check if a rumor is known.
    pub fn knows(&self, id: RumorId) -> bool {
        self.known_rumors.contains(&id)
    }

    /// Number of known rumors.
    pub fn len(&self) -> usize {
        self.known_rumors.len()
    }

    /// Is the store empty?
    pub fn is_empty(&self) -> bool {
        self.known_rumors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rumor_creation() {
        let r = Rumor::new(1, vec![42], 0, 0);
        assert_eq!(r.id, 1);
        assert_eq!(r.ttl, 10);
    }

    #[test]
    fn test_rumor_ttl_decrement() {
        let mut r = Rumor::new(1, vec![], 0, 0);
        r.ttl = 3;
        assert!(r.tick()); // ttl=2
        assert!(r.tick()); // ttl=1
        assert!(!r.tick()); // ttl=0
    }

    #[test]
    fn test_store_add_rumor() {
        let mut store = RumorStore::new();
        let r = Rumor::new(1, vec![1], 0, 0);
        assert!(store.add(r));
        assert!(!store.add(Rumor::new(1, vec![1], 0, 0))); // duplicate
    }

    #[test]
    fn test_store_tick_removes_expired() {
        let mut store = RumorStore::new();
        let mut r = Rumor::new(1, vec![], 0, 0);
        r.ttl = 1;
        store.add(r);
        store.tick();
        assert!(store.get_pending().is_empty());
        assert!(store.knows(1)); // still known
    }

    #[test]
    fn test_mark_delivered() {
        let mut store = RumorStore::new();
        store.add(Rumor::new(1, vec![], 0, 0));
        store.add(Rumor::new(2, vec![], 0, 0));
        store.mark_delivered(1);
        assert_eq!(store.get_pending().len(), 1);
    }

    #[test]
    fn test_knows_check() {
        let mut store = RumorStore::new();
        store.add(Rumor::new(42, vec![], 0, 0));
        assert!(store.knows(42));
        assert!(!store.knows(99));
    }
}
