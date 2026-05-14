# Security Policy

Orchestrator is built with a "Zero Trust" approach toward AI-generated code and commands.

## Threat Model

1. **Malicious Tool Use**: Prevent LLMs from executing destructive commands without user approval.
2. **Context Leakage**: Ensure only specified files are injected into the LLM context.
3. **Sandbox Escape**: Use Sandbox (UDS-based sandbox) to isolate execution from the host.

## Enforcement Mechanisms

- **Style Gates**: 
  - `Casual`: No tool access.
  - `Context`: Read-only/Search only.
  - `Build`: Full access, but requires Sandbox policy approval.
- **Diff Review**: Every filesystem change must be visually accepted (`[a]`) by the user.
- **UDS Isolation**: Communication with sidecars happens over `/tmp/ruthenlabs/*.sock`, preventing network-based attacks on the control plane.
- **Token Limits**: Automated compaction prevents context-overflow attacks.

## Policy Configuration

User policies are stored in `~/.config/sandbox/policies/*.toml` and synced via Orchestrator's `/mode` command.
