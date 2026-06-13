# Gossip Protocol

**A Rust library implementing the SWIM gossip protocol for decentralized cluster membership and failure detection** — epidemic-style information propagation with configurable fan-out, intervals, and suspicion mechanics.

## Why It Matters

The SWIM (Scalable Weakly-consistent Infection-style Process Group Membership) protocol powers membership management in HashiCorp Consul, Nomad, and memberlist (used by Kubernetes' cluster-autoscaler). Unlike heartbeat-based protocols, SWIM piggybacks membership updates on periodic probes, achieving both failure detection and dissemination in a single round. The protocol scales to thousands of nodes with bounded bandwidth per node. In the SuperInstance ecosystem, this protocol maintains the edge Worker fleet topology, detecting crashed Workers and propagating deployment state changes.

## How It Works

Each node maintains a local membership list and runs a periodic protocol cycle (default ~1 second): (1) **Probe** — select a random member, send a ping, wait for ack. On failure, dispatch indirect pings through k random peers. (2) **Suspect** — if all probes fail, mark the node as suspect with an incarnation number. (3) **Disseminate** — piggyback the membership update (alive, suspect, or dead) on the next gossip message sent to peers. Updates spread epidemically: after `O(log N)` rounds, all nodes have the update with high probability.

## Quick Start

```rust
// API surface under development — the crate currently provides
// foundational types for protocol message handling.
use gossip_protocol::add;

fn main() {
    assert_eq!(add(2, 2), 4);
}
```

## API

| Function | Description |
|---|---|
| `add(left, right)` | Placeholder — full protocol API under development |

## Architecture Notes

Core of the SuperInstance gossip stack: `gossip-protocol` (wire protocol + state machine), `gossip-member` (membership records), `gossip-ping` (probes), `gossip-seed` (bootstrap nodes), `gossip-suspicion` (timeout handling). See the [Architecture Guide](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
