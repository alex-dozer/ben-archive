# BEE - Ben Escrow Emissary

## Purpose

BEE acts as Ben's neutral emissary between external applications and the Ben ecosystem.  
It is the escrow agent that receives socket handoffs, validates integrity, and certifies the health of the data lanes (Diagnostic, Event, Control) before Ben accepts any telemetry.

## Philosophy

Trust must be earned, verified, and recorded.  
BEE brokers that trust by performing a short diplomatic exchange to prove both identity and communication fidelity without ever touching the adopter's system internals.

## Lifecycle

1. **Spawn**  
    Triggered when a new client connects to `/run/ben/handoff.sock`.
    
2. **Greeting**  
    Performs the HELLO -> CHALLENGE -> READY handshake, exchanging keys and spec fingerprints.
    
3. **Proof of Wire**  
    Executes deterministic diagnostics across all three FD lanes: ping, framing, backpressure, control signature, and contingency sandbox tests.
    
4. **Certification**  
    Emits a signed handoff certificate summarizing the session's RTTs, watermark behavior, and policy conformity. Stored in ClickHouse for audit.
    
5. **Observation**  
    Monitors for early anomalies within a short grace period after certification.
    
6. **Retirement**  
    Closes descriptors, writes final narrative event, and self-terminates.
    

## Behavioral Notes

- Each BEE instance is ephemeral; spawned per handoff, owned by the parent sidecar or benscrow manager.
    
- Operates under the Principle of Diplomatic Immunity: it cannot modify client state, only observe and verify.
    
- All actions are signed and timestamped; no silent failures.
    
- On failure, emits a Refusal Certificate detailing what broke and suggested remediation.
    

## Output Artifacts

- `handoff_certificate` (JSON and ClickHouse row)
    
- Human log (narrative form)
    
- Metrics: RTTs, queue behavior, watermark transitions, sandbox test results
    

### Example Narrative Log

`[bee#42] Greeting client pid=7734 caps=diag,event,control [bee#42] Proof of Wire complete: RTT=0.4 ms, framing OK [bee#42] Certification issued sha256:d1e9... [bee#42] Retired gracefully after 180 s`

## Tagline

Ben Escrow Emissary: verifying the line between observation and trust.

## Repository Layout (proposed)

`bee/  ├─ src/  │   ├─ main.rs           // Entrypoint, spawns per-handoff worker  │   ├─ handoff.rs        // Unix socket accept loop and greeter  │   ├─ handshake.rs      // HELLO -> CHALLENGE -> READY finite state machine  │   ├─ diagnostics.rs    // Proof-of-Wire tests  │   ├─ certificate.rs    // Signing, HMAC, ClickHouse emitters  │   ├─ narrative.rs      // Human-readable storytelling logs  │   ├─ policy.rs         // Grace period rules and thresholds  │   └─ shutdown.rs       // Retirement sequence and cleanup  ├─ Cargo.toml  └─ README.md`

## Design Principles

1. Immutable stance: BEE never changes external state.
    
2. Deterministic diagnostics: tests must be reproducible.
    
3. Signed narrative: every log line is verifiable.
    
4. Short life span: each instance terminates after its session.
    
5. Transparency: all errors produce an explicit reason and remediation.
