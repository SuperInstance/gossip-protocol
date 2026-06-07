# gossip-protocol

Gossip protocol simulation: rumor spreading, anti-entropy, scuttlebutt, membership.

## Features

- **Rumor Mongering** — TTL-based rumor spreading with fanout
- **Anti-Entropy** — Full state synchronization with digest exchange
- **Membership** — Suspicion-based failure detection with incarnation numbers
- **Fanout** — Optimal fanout calculation and deterministic peer selection
- **Convergence** — Simulated network with convergence metrics

## Modules

| Module | Description |
|--------|-------------|
| `rumor` | Rumor mongering with TTL and fanout |
| `anti_entropy` | Anti-entropy synchronization |
| `membership` | Suspicion-based failure detection |
| `fanout` | Fanout calculation and peer selection |
| `convergence` | Convergence metrics and simulation |

## Usage

```rust
use gossip_protocol::convergence::SimulatedNetwork;
use gossip_protocol::rumor::Rumor;

let mut net = SimulatedNetwork::new(10);
net.inject_rumor(0, Rumor::new(1, vec![42], 0, 0));
let metrics = net.run_until_converged(3, 50);
println!("Converged in {} rounds", metrics.rounds);
```

## Testing

```bash
cargo test    # 31 tests
cargo clippy  # zero warnings
```

## License

MIT
