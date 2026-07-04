---
description: Mark review notes as resolved after they've been addressed
---

The user wants review notes marked as resolved: $ARGUMENTS

1. Call the thruline MCP tool `get_review_queue` to see open notes.
2. If the user gave note ids, resolve those with `resolve_review_note`. If they said "all" or gave no ids, resolve every note that has genuinely been addressed in this session — and say which ones you are NOT resolving because they weren't addressed.
3. Confirm compactly: which ids were resolved, which remain open.
