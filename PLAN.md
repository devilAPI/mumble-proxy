# Implementation plan

## Topology
```
 TS3 clients ──UDP──▶ official TS3 server (bundled, unmodified, "front door")
                          ▲  ▲ ServerQuery (admin: channel sync, anti-flood setup, observe)
                          │  │ TS3 client connections (tsclientlib): one puppet per Mumble user,
                          │  │ plus one "listener" per TS3-occupied channel if no puppet is there
                     ┌────┴──┴─────────────┐
                     │   mumble-proxy      │  per-session tasks, isolated
                     └────┬────────────────┘
                          │ one Mumble connection per TS3 user (TLS + client cert per TS3 UID)
                          ▼
                    unmodified Mumble server  (authoritative for channels)
```

## Core rules
- **Mumble is authoritative for the channel tree.** The gateway creates, renames, moves and deletes TS3 channels
  through ServerQuery so they mirror Mumble. Mapping rules come from config.
- **Loop prevention.** Gateway-owned sessions are tracked on both sides: TS3 puppets by client id, and
  Mumble avatars by session id plus a cert-hash set. They are never mirrored back.
- **Isolation.** Each mirrored user is a tokio task with its own connection. A panic or error ends only that
  task, and a supervisor reconnects it with backoff. Translation logic is pure (`translate/` module, no I/O)
  so it can be unit-tested.
- **Identity.** One self-signed cert per TS3 unique id under `state/certs/<sha256(uid)>.pem`, created on first
  sight and reused afterwards.

## Modules
- `config` – TOML settings: listen/upstream addresses, TLS, ServerQuery credentials, session cap, name affixes, collision policy, channel rules
- `translate::channels` – Mumble tree ⇄ TS3 tree id mapping and diffing into ops (pure)
- `translate::users` – nickname mapping, prefix/suffix, collision resolution, state flags (pure)
- `translate::text` – Mumble HTML ⇄ TS3 BBCode/plain, length limits (pure)
- `mumble::session` – one Mumble client: TLS, ping, events in, commands out
- `ts3::query` – ServerQuery client (channel sync, settings, observe events)
- `ts3::puppet` – one tsclientlib connection per Mumble user
- `bridge` – supervisor: session cap, lifecycle, routing, SIGINT/SIGTERM shutdown
- Phase 2: `audio` (jitter buffer, Opus passthrough or transcode)

## Phase 1 milestones (each ends with cargo build, clippy and test)
1. Config, logging, supervisor skeleton, shutdown, and session cap with a log line when the cap is hit
2. Pure translation layer and its unit tests (channels, users, collisions, state transitions)
3. Mumble session: connect, sync, reconnect, per-UID certs
4. ServerQuery: observe TS3 clients, mirror the Mumble channel tree into TS3, anti-flood setup
5. TS3 user → Mumble avatar lifecycle (join, leave, nickname)
6. Mumble user → TS3 puppet lifecycle
7. Text chat both ways (channel and server; private messages move to Phase 2 as specified)
8. Manual test plan plus an integration run against `dev/compose.dev.yml`
