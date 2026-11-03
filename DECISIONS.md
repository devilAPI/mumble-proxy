# Decisions log

## 2026-11-03 – D1: Rust, reuse ReSpeak tsproto (MIT/Apache-2.0)
tsproto is permissively licensed and covers packet codec, crypto and license parsing. TS3AudioBot (OSL-3.0, copyleft)
is used only as a reference and not copied. ts3j (Apache-2.0) serves as a secondary reference.

## 2026-11-03 – D2: No PoC of the TS3-server frontend
The stock TS3 client (3.2 and newer) needs a license chain rooted at TeamSpeak's hardcoded root key, and the
server must hold the matching private key. Getting that key legitimately is not possible. TeaSpeak's workaround
(`protocol_key.txt`) is exactly the leaked/extracted-key route the requirements forbid. Building a PoC against
tsclientlib with a custom `root=` would prove nothing about stock clients. See FEASIBILITY.md §3–4.

## 2026-11-03 – D4: Publicly known client_version_sign is acceptable
User approved using the public version-sign triple that TS3 bots use (non-commercial project).

## 2026-11-03 – D3: Recommend pivot to puppeting bridge with a real TS3 server (pending user approval)
See FEASIBILITY.md §5. Phase 1 stays blocked until the user decides.

## 2026-11-03 – D5: Architecture = puppeting bridge with a bundled TS3 "front door" (user-approved)
TS3 users connect to an official, unmodified TS3 server shipped in docker-compose. It is only a front door
whose channel tree mirrors Mumble. The gateway is a TS3 *client* (tsproto) plus Mumble clients.
- TS3 user  -> one dedicated Mumble connection (individual Mumble user, stable cert per TS3 UID).
- Mumble user -> one dedicated TS3 puppet client (user chose "one TS3 puppet each").
Consequences: both directions need loop prevention (never mirror a puppet back), and
TS3 slot usage = TS3 users + Mumble users + gateway helper clients.

## 2026-11-03 – D6: Libraries
- TS3: `tsclientlib` from git (pinned rev). crates.io 0.2.0 dates from 2021, git master is maintained. Verified against the official server.
- Mumble: `mumble-protocol-2x` 0.6 (codec only), with `native-tls` (OpenSSL) for client certs and `rcgen` for cert generation.
- Opus (Phase 2): `opus` crate as requested.

## 2026-11-03 – D7: Gateway uses ServerQuery admin on the bundled TS3 server
It is needed to mirror the channel tree, to observe clients cheaply, and to raise anti-flood limits for the
gateway IP. Credentials come only from config or environment, never hardcoded.
