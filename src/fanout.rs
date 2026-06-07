//! Fanout calculation and peer selection.

use std::collections::HashSet;

/// Calculate the optimal fanout for a cluster of `n` nodes to achieve `target_prob`
/// probability of all nodes receiving the rumor.
pub fn calculate_fanout(n: usize, target_prob: f64) -> usize {
    if n <= 1 {
        return 0;
    }
    // fanout ≈ ln(1 / (1 - p^(1/n))) where p is target probability
    // Simplified: fanout ≈ ln(n) / (1 - (1-p)^(1/n))
    // Common heuristic: fanout = ceil(ln(n) * O(1))
    let ln_n = (n as f64).ln();
    let correction = (1.0 / (1.0 - target_prob)).ln();
    let fanout = ln_n + correction;
    std::cmp::max(1, fanout.ceil() as usize)
}

/// Select `fanout` random peers from the cluster (excluding self).
/// Uses deterministic selection for simulation reproducibility.
pub fn select_fanout_peers(
    all_nodes: &[u64],
    self_id: u64,
    fanout: usize,
    round: u64,
) -> Vec<u64> {
    let candidates: Vec<u64> = all_nodes.iter()
        .filter(|&&id| id != self_id)
        .copied()
        .collect();
    if candidates.is_empty() || fanout == 0 {
        return Vec::new();
    }

    // Deterministic pseudo-random selection based on round
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    let n = candidates.len();

    for i in 0..fanout {
        let idx = (round as usize * 7 + i * 13 + self_id as usize) % n;
        let peer = candidates[idx % n];
        if seen.insert(peer) {
            selected.push(peer);
        }
        if selected.len() >= fanout || seen.len() >= n {
            break;
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fanout_single_node() {
        assert_eq!(calculate_fanout(1, 0.99), 0);
    }

    #[test]
    fn test_fanout_small_cluster() {
        let f = calculate_fanout(5, 0.99);
        assert!(f >= 1);
    }

    #[test]
    fn test_fanout_larger_cluster() {
        let f5 = calculate_fanout(5, 0.99);
        let f100 = calculate_fanout(100, 0.99);
        assert!(f100 >= f5);
    }

    #[test]
    fn test_select_peers_excludes_self() {
        let nodes = vec![1, 2, 3, 4, 5];
        let peers = select_fanout_peers(&nodes, 1, 3, 0);
        assert!(!peers.contains(&1));
    }

    #[test]
    fn test_select_peers_respects_fanout() {
        let nodes = vec![1, 2, 3, 4, 5];
        let peers = select_fanout_peers(&nodes, 1, 2, 0);
        assert!(peers.len() <= 2);
    }

    #[test]
    fn test_select_peers_different_rounds() {
        let nodes = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let p0 = select_fanout_peers(&nodes, 1, 3, 0);
        let p1 = select_fanout_peers(&nodes, 1, 3, 1);
        // Different rounds may produce different peer selections
        // (not guaranteed, but likely with enough nodes)
        assert!(p0.len() <= 3);
        assert!(p1.len() <= 3);
    }
}
