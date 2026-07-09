# Plan 065 Remaining: Agent-Session Surface

## Audit Verdict

Core/API/UI implementation is mostly landed. A regression test now pins that
`execute_tool` spans with `shell.command` remain tool steps unless the tool is
actually `shell_command` or has process attrs. Remaining item is live producer
pairing proof.

## Remaining Work

- [ ] Run the current agent producer and collect a run with `invoke_agent`,
  generic `execute_tool`, and shell-command steps.
- [ ] Verify `agentSession(runId)` classifies steps correctly and sums token
  usage/error count.
- [ ] Record native GreptimeDB trace evidence and Parallax UI evidence.

## Remove When

- Live agent producer pairing proof is recorded.
