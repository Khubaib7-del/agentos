---
description: Process an interrupt correction as a review, then continue the interrupted work
allowed-tools: Bash(thruline note:*)
---

The user interrupted you mid-task to give a course correction: $ARGUMENTS

Treat this as a review of your work so far, in this exact order:

1. Log it so it survives the session: run `thruline note "<the correction, quoted>"`.
2. Audit everything you already did in this session against the correction. List anything that now conflicts with it — files, code, decisions.
3. Fix the conflicts first, before any new work.
4. State in one short paragraph what the correction changed.
5. Then — and only then — continue the task you were doing when interrupted.

Do not skip the audit even if you believe nothing conflicts; say so explicitly after checking.
