//! Anti-entropy: full state synchronization between nodes.

use crate::rumor::{Rumor, RumorId, RumorStore};

/// Digest of a node's rumor state for comparison.
#[derive(Debug, Clone)]
pub struct Digest {
    pub node_id: u64,
    pub known_ids: Vec<RumorId>,
}

impl Digest {
    pub fn new(node_id: u64, store: &RumorStore) -> Self {
        Self { node_id, known_ids: store.known_rumors.clone() }
    }

    /// Rumors that `other` has but we don't.
    pub fn missing_from(&self, other: &Digest) -> Vec<RumorId> {
        other.known_ids.iter()
            .filter(|id| !self.known_ids.contains(id))
            .copied()
            .collect()
    }
}

/// Anti-entropy protocol for full sync.
#[derive(Debug)]
pub struct AntiEntropy {
    pub node_id: u64,
    pub store: RumorStore,
}

impl AntiEntropy {
    pub fn new(node_id: u64) -> Self {
        Self { node_id, store: RumorStore::new() }
    }

    /// Compute digest for exchange.
    pub fn digest(&self) -> Digest {
        Digest::new(self.node_id, &self.store)
    }

    /// Compute what we need from the other node's digest.
    pub fn compute_diff(&self, other_digest: &Digest) -> Vec<RumorId> {
        let my_digest = self.digest();
        my_digest.missing_from(other_digest)
    }

    /// Apply received rumors.
    pub fn apply_rumors(&mut self, rumors: Vec<Rumor>) -> usize {
        let mut count = 0;
        for r in rumors {
            if self.store.add(r) {
                count += 1;
            }
        }
        count
    }

    /// Full sync with another node: exchange digests and reconcile.
    pub fn sync_with(&mut self, other: &mut AntiEntropy) -> (usize, usize) {
        let my_digest = self.digest();
        let other_digest = other.digest();

        let i_need: Vec<RumorId> = my_digest.missing_from(&other_digest);
        let they_need: Vec<RumorId> = other_digest.missing_from(&my_digest);

        let my_rumors: Vec<Rumor> = they_need.iter()
            .filter_map(|id| self.store.pending.iter().find(|r| r.id == *id).cloned())
            .collect();
        let their_rumors: Vec<Rumor> = i_need.iter()
            .filter_map(|id| other.store.pending.iter().find(|r| r.id == *id).cloned())
            .collect();

        let applied_here = self.apply_rumors(their_rumors);
        let applied_there = other.apply_rumors(my_rumors);
        (applied_here, applied_there)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_creation() {
        let mut ae = AntiEntropy::new(1);
        ae.store.add(Rumor::new(10, vec![], 1, 0));
        let digest = ae.digest();
        assert_eq!(digest.node_id, 1);
        assert!(digest.known_ids.contains(&10));
    }

    #[test]
    fn test_missing_from() {
        let mut ae1 = AntiEntropy::new(1);
        let mut ae2 = AntiEntropy::new(2);
        ae1.store.add(Rumor::new(1, vec![], 1, 0));
        ae2.store.add(Rumor::new(2, vec![], 2, 0));
        let d1 = ae1.digest();
        let d2 = ae2.digest();
        assert!(d1.missing_from(&d2).contains(&2));
        assert!(d2.missing_from(&d1).contains(&1));
    }

    #[test]
    fn test_full_sync() {
        let mut ae1 = AntiEntropy::new(1);
        let mut ae2 = AntiEntropy::new(2);
        ae1.store.add(Rumor::new(1, vec![1], 1, 0));
        ae2.store.add(Rumor::new(2, vec![2], 2, 0));
        let (a, b) = ae1.sync_with(&mut ae2);
        assert_eq!(a, 1); // ae1 got rumor 2
        assert_eq!(b, 1); // ae2 got rumor 1
    }

    #[test]
    fn test_no_sync_needed() {
        let mut ae1 = AntiEntropy::new(1);
        let mut ae2 = AntiEntropy::new(2);
        ae1.store.add(Rumor::new(1, vec![], 1, 0));
        ae2.store.add(Rumor::new(1, vec![], 1, 0));
        let (a, b) = ae1.sync_with(&mut ae2);
        assert_eq!(a, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_apply_rumors_dedup() {
        let mut ae = AntiEntropy::new(1);
        ae.store.add(Rumor::new(1, vec![], 1, 0));
        let count = ae.apply_rumors(vec![Rumor::new(1, vec![], 1, 0)]);
        assert_eq!(count, 0); // already known
    }
}
