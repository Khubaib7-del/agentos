---
description: Record a project decision (add --lock if the user marks it final)
allowed-tools: Bash(thruline decide:*)
---

Record the decision the user stated: run `thruline decide "<decision>"` with `--why "<rationale>"` if they gave a reason, and `--lock` only if they indicated it is final/locked. Parse these from: $ARGUMENTS

Confirm in one short line what was recorded and whether it is locked.
