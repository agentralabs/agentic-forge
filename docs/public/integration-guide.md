# Integration Guide

AgenticForge integrates with the Agentra Labs sister ecosystem through
trait-based bridges.

## Available Bridges

| Bridge          | Sister            | Purpose                            |
|-----------------|-------------------|------------------------------------|
| AegisBridge     | AgenticAegis      | Security policy enforcement        |
| EvolveBridge    | AgenticEvolve     | Schema evolution tracking          |
| VeritasBridge   | AgenticVeritas    | Verification and proof             |
| MemoryBridge    | AgenticMemory     | Persistent memory storage          |
| IdentityBridge  | AgenticIdentity   | Identity and auth integration      |
| TimeBridge      | AgenticTime       | Temporal event correlation         |
| CognitionBridge | AgenticCognition  | Reasoning and reflection           |
| CommBridge      | AgenticComm       | Inter-agent communication          |
| PlanningBridge  | AgenticPlanning   | Plan coordination                  |
| RealityBridge   | AgenticReality    | Ground truth validation            |

## Bridge Pattern

All bridges use NoOp defaults for standalone operation:

```rust
pub trait MemoryBridge: Send + Sync {
    fn store_blueprint(&self, _id: &str, _data: &[u8]) -> Result<(), BridgeError> {
        Ok(()) // NoOp default
    }
}
```

## MCP Integration

Start the MCP server and connect it to any MCP-compatible client:

```bash
aforge serve --mode stdio
```

The server exposes 15 tools via JSON-RPC over stdio transport.

## SDK Dependency

AgenticForge depends on `agentic-sdk v0.2.0` for shared types and bridge
trait definitions.
