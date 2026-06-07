//! Convergence metrics and gossip simulation.

use crate::rumor::{Rumor, RumorStore};
use crate::fanout::select_fanout_peers;

/// Metrics for gossip convergence.
#[derive(Debug, Clone)]
pub struct ConvergenceMetrics {
    pub rounds: u64,
    pub total_messages: u64,
    pub nodes_informed: usize,
    pub total_nodes: usize,
}

impl ConvergenceMetrics {
    pub fn convergence_ratio(&self) -> f64 {
        if self.total_nodes == 0 { return 1.0; }
        self.nodes_informed as f64 / self.total_nodes as f64
    }

    pub fn is_fully_converged(&self) -> bool {
        self.nodes_informed == self.total_nodes
    }
}

/// Simulated gossip network.
pub struct SimulatedNetwork {
    pub nodes: Vec<GossipNode>,
    pub all_ids: Vec<u64>,
}

/// A node in the simulated network.
#[derive(Debug, Clone)]
pub struct GossipNode {
    pub id: u64,
    pub store: RumorStore,
}

impl GossipNode {
    pub fn new(id: u64) -> Self {
        Self { id, store: RumorStore::new() }
    }
}

impl SimulatedNetwork {
    pub fn new(node_count: usize) -> Self {
        let nodes: Vec<GossipNode> = (0..node_count)
            .map(|i| GossipNode::new(i as u64))
            .collect();
        let all_ids: Vec<u64> = (0..node_count as u64).collect();
        Self { nodes, all_ids }
    }

    /// Inject a rumor at a specific node.
    pub fn inject_rumor(&mut self, node_idx: usize, rumor: Rumor) {
        self.nodes[node_idx].store.add(rumor);
    }

    /// Run one round of gossip.
    pub fn round(&mut self, fanout: usize) -> u64 {
        let mut messages = 0u64;
        // Collect what each node will send
        let mut deliveries: Vec<(usize, Vec<Rumor>)> = Vec::new();

        let current_round = self.nodes.iter()
            .map(|n| n.store.known_rumors.len() as u64)
            .max()
            .unwrap_or(0);

        for node in &self.nodes {
            if node.store.get_pending().is_empty() {
                continue;
            }
            let peers = select_fanout_peers(&self.all_ids, node.id, fanout, current_round);
            let rumors: Vec<Rumor> = node.store.get_pending().to_vec();
            for peer_id in peers {
                if let Some(peer_idx) = self.nodes.iter().position(|n| n.id == peer_id) {
                    deliveries.push((peer_idx, rumors.clone()));
                    messages += 1;
                }
            }
        }

        // Apply deliveries
        for (idx, rumors) in deliveries {
            for r in rumors {
                self.nodes[idx].store.add(r);
            }
        }

        // Tick all stores
        for node in &mut self.nodes {
            node.store.tick();
        }

        messages
    }

    /// Run until convergence or max rounds.
    pub fn run_until_converged(&mut self, fanout: usize, max_rounds: u64) -> ConvergenceMetrics {
        let total = self.nodes.len();
        let mut rounds = 0u64;
        let mut total_messages = 0u64;

        for r in 0..max_rounds {
            rounds = r + 1;
            total_messages += self.round(fanout);

            // Check if all nodes know all rumors
            let all_rumors: std::collections::HashSet<u64> = self.nodes.iter()
                .flat_map(|n| n.store.known_rumors.iter().copied())
                .collect();

            let informed = self.nodes.iter()
                .filter(|n| all_rumors.iter().all(|rid| n.store.knows(*rid)))
                .count();

            if informed == total {
                return ConvergenceMetrics {
                    rounds,
                    total_messages,
                    nodes_informed: informed,
                    total_nodes: total,
                };
            }
        }

        let informed = self.nodes.iter()
            .filter(|n| !n.store.known_rumors.is_empty())
            .count();

        ConvergenceMetrics { rounds, total_messages, nodes_informed: informed, total_nodes: total }
    }

    /// Get convergence ratio (fraction of nodes knowing all rumors).
    pub fn convergence_ratio(&self) -> f64 {
        let all_rumors: std::collections::HashSet<u64> = self.nodes.iter()
            .flat_map(|n| n.store.known_rumors.iter().copied())
            .collect();
        if all_rumors.is_empty() { return 1.0; }
        let informed = self.nodes.iter()
            .filter(|n| all_rumors.iter().all(|rid| n.store.knows(*rid)))
            .count();
        informed as f64 / self.nodes.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_rumor_convergence() {
        let mut net = SimulatedNetwork::new(5);
        net.inject_rumor(0, Rumor::new(1, vec![42], 0, 0));
        let metrics = net.run_until_converged(3, 20);
        assert!(metrics.is_fully_converged());
    }

    #[test]
    fn test_convergence_metrics() {
        let mut net = SimulatedNetwork::new(10);
        net.inject_rumor(0, Rumor::new(1, vec![], 0, 0));
        let metrics = net.run_until_converged(3, 50);
        assert!(metrics.rounds > 0);
        assert!(metrics.total_messages > 0);
    }

    #[test]
    fn test_multiple_rumors() {
        let mut net = SimulatedNetwork::new(8);
        net.inject_rumor(0, Rumor::new(1, vec![1], 0, 0));
        net.inject_rumor(4, Rumor::new(2, vec![2], 4, 0));
        let metrics = net.run_until_converged(3, 30);
        assert!(metrics.convergence_ratio() > 0.8);
    }

    #[test]
    fn test_no_rumors_converged() {
        let net = SimulatedNetwork::new(5);
        assert_eq!(net.convergence_ratio(), 1.0);
    }

    #[test]
    fn test_fanout_affects_speed() {
        let mut net1 = SimulatedNetwork::new(20);
        net1.inject_rumor(0, Rumor::new(1, vec![], 0, 0));
        let m1 = net1.run_until_converged(2, 100);

        let mut net2 = SimulatedNetwork::new(20);
        net2.inject_rumor(0, Rumor::new(1, vec![], 0, 0));
        let m2 = net2.run_until_converged(5, 100);

        // Higher fanout should converge in fewer rounds
        assert!(m2.rounds <= m1.rounds);
    }

    #[test]
    fn test_metrics_ratio() {
        let m = ConvergenceMetrics { rounds: 5, total_messages: 20, nodes_informed: 8, total_nodes: 10 };
        assert!((m.convergence_ratio() - 0.8).abs() < 0.01);
    }
}
