# Gossip Protocol

A **epidemic-style information dissemination protocol** that spreads membership updates, application data, and control messages across a distributed cluster with O(log N) convergence — no central coordinator, no broadcast storms, gracefully handles partitions and node churn.

## Why It Matters

Centralized coordination services (ZooKeeper, etcd) introduce a single point of failure and a scaling bottleneck. Gossip protocols sidestep both: every node is equal, information spreads like a virus through periodic peer-to-peer exchanges, and the system is eventually consistent with probabilistic convergence guarantees. In a 1000-node cluster, a piece of information reaches all nodes in ~14 rounds (log₂(1000) ≈ 10, with safety margin). This library implements the full gossip stack: state dissemination, anti-entropy synchronization, and push-pull fanout. For SuperInstance's distributed agent fleet, gossip is the substrate that lets agents discover each other without a central registry.

## How It Works

**Gossip exchange**: Every T milliseconds (typically 1s), each node:
1. Picks a random peer (or round-robin through the member list)
2. Sends a digest of its current state (a Merkle-tree root or a list of `(key, version)` pairs)
3. The peer responds with entries where its version is newer (push) and requests entries where the digest indicates it's behind (pull)

**Anti-entropy**: To prevent information loss, nodes periodically (every T_reconcile, typically 30s) exchange full state snapshots rather than just deltas. This is the "rumor mongering with anti-entropy" hybrid from Demers et al.

**Convergence analysis**: Consider a single piece of information introduced at one node. In each round, each informed node gossips to one random peer. The number of informed nodes follows:
```
dI/dt ≈ β · I · (N - I) / N
```
where β is the fanout (peers per round), N is cluster size, and I is informed nodes. This logistic growth model gives convergence time of O(log N / β) rounds with probability 1 - 1/N.

**Message types**:
- **Alive**: `{node, inc}` — node is alive at incarnation `inc`
- **Suspect**: `{node, inc}` — node suspected dead at incarnation `inc`
- **Confirm**: `{node, inc}` — node confirmed dead
- **Refute**: `{node, inc+1}` — suspected node refutes with higher incarnation

**State reconciliation**: Two nodes meeting for the first time exchange full membership tables. Each entry is merged by `(incarnation, state_precedence)` — the core arbitration rule that prevents stale or conflicting updates from corrupting the membership.

**Complexity**:
- Per gossip round: O(fanout × message_size) bandwidth per node
- Convergence: O(log N) rounds with fanout=1, O(log N / fanout) with higher fanout
- Memory: O(N) per node (full member table)
- CPU: O(N) per round for state reconciliation

## Quick Start

```rust
use gossip_protocol::{GossipNode, GossipConfig};

let config = GossipConfig::new()
    .bind_addr("0.0.0.0:7946")
    .probe_interval_ms(1000)
    .fanout(3);

let mut node = GossipNode::new("node-1", config);
node.start().await;

// Broadcast application data
node.broadcast("cluster-config", b"v2");

// Leave gracefully
node.leave().await;
```

## API

| Type | Description |
|------|-------------|
| `GossipNode::new(id, config)` | Create and bind a gossip node |
| `GossipConfig` | Configuration (bind addr, fanout, intervals) |
| `.start()` | Begin gossip loop (async) |
| `.broadcast(key, value)` | Disseminate a key-value pair to all nodes |
| `.leave()` | Announce departure and shut down cleanly |
| `.members()` | Current membership list |

## Architecture Notes

Gossip Protocol is the communication backbone of the SuperInstance distributed layer. It orchestrates gossip-member (state), gossip-ping (liveness), gossip-suspicion (failure handling), and gossip-seed (bootstrapping). The protocol's O(log N) convergence is what enables the fleet to scale — adding nodes doesn't increase per-node coordination cost. In **γ + η = C**, gossip converts coordination from explicit (γ) to reflexive (η): information spreads automatically. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Demers, A. et al. "Epidemic Algorithms for Replicated Database Maintenance," PODC (1987).
- Das, A. et al. "SWIM: Scalable Weakly-consistent Infection-style Membership," DSN (2002).
- Jelasity, M. et al. "Gossip-Based Peer Sampling," ACM TOCS (2007).

## License

MIT
