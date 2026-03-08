# Future Development Ideas

## VSCode Extension — Mobile Code Bridge

**Goal**: Build a custom VSCode extension that replaces the official Claude Code extension, with built-in mobile two-way interaction.

### Architecture
```
User (VSCode) ──┐
                 ├── Our Extension ── claude CLI (stream-json) ── Claude API
User (Mobile) ──┘        │
                    Relay server
                   (Happy relay or custom)
```

### How it works
1. Extension spawns `claude --print --input-format stream-json --output-format stream-json`
2. Parses JSON stream → displays in VSCode webview (nice UI, Vietnamese, images)
3. Simultaneously relays messages to mobile via relay
4. Mobile sends message → relay → extension receives → feeds into claude stdin
5. Claude response → shown on both VSCode webview and mobile

### Why this works
- We own the entire message pipeline → two-way sync is possible
- Claude CLI handles all tool use, permissions, file editing — we just need UI + relay
- Not dependent on Anthropic's extension
- Supports Vietnamese, images, rich UI in webview

### Estimated effort
- VSCode extension scaffold + webview UI: 1-2 days
- Claude CLI stream-json integration: 1 day
- Mobile relay (Happy relay or custom): 1-2 days
- Polish: 1-2 days

### Marketplace considerations
- Use neutral name (e.g., "Mobile Code Bridge", "CodeSync Mobile") to avoid trademark issues
- Mark as "unofficial/third-party"
- Happy Coder precedent: third-party Claude Code tools exist on stores without issues

### Why not integrate with existing Claude Code extension
- VSCode extension uses stream-json SDK protocol internally
- Happy needs PTY (terminal I/O) to sync — can't capture JSON stream
- `claudeProcessWrapper` setting spawns .exe but protocol mismatch prevents sync
- No API to inject messages into a running Claude Code extension session
