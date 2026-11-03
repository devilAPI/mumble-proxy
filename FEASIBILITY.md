# Phase 0 – Feasibility: TS3-protocol frontend for a Mumble gateway

**Date:** 2026-11-03
**Verdict:** **NO-GO** for the original architecture (a TS3-protocol server that stock TS3 clients connect to),
under the project's own constraint: no leaked, extracted or forged TeamSpeak keys.
**Recommendation:** pivot to a *puppeting bridge* in front of a real, unmodified TeamSpeak 3 server (see §5).

---

## 1. What was researched

| Source | License | What it covers |
|---|---|---|
| [ReSpeak/tsdeclarations `ts3protocol.md`](https://github.com/ReSpeak/tsdeclarations/blob/master/ts3protocol.md) | MIT/Apache-2.0 | Reverse-engineered spec: packet framing, Init1 puzzle, EAX encryption, `clientinitiv` / `initivexpand2` / `clientek`, license chain |
| [ReSpeak/tsclientlib (`tsproto`)](https://github.com/ReSpeak/tsclientlib) | MIT/Apache-2.0 | Rust **client** implementation: packet codec, resend/ack, crypto, license parsing **and** license building / private-key derivation helpers |
| [Splamy/TS3AudioBot (TS3Client)](https://github.com/Splamy/TS3AudioBot) | OSL-3.0 (copyleft; not reusable in an MIT/Apache crate without care) | C# **client** |
| [Manevolent/ts3j](https://github.com/Manevolent/ts3j) | Apache-2.0 | Java **client** |
| [tsclientlib issue #12](https://github.com/ReSpeak/tsclientlib/issues/12), [TeaSpeak forum](https://forum.teaspeak.de/index.php?threads/compatibility-ts3-client-and-teaspeak-server.1190/), [noirth tutorial](https://noirth.com/threads/connect-with-ts-clients-3-2-0-to-teaspeak-servers.7107/) | – | How TeaSpeak, the only notable third-party TS3 server, handled stock clients |

All the open-source work is **client-side**. No permissively licensed server-side handshake exists. Most of the
low-level machinery is symmetric and could be reused, though: packet codec, EAX, the Init1 RSA puzzle
(the server just needs to generate `x, n, level` and check `y`), resend/ack and command parsing. The handshake
is the only real blocker.

## 2. How the handshake works (server's point of view)

1. **Init1, steps 0–4 (low level).** The server sends an RSA time-lock puzzle (`x, n, level`), and the client returns
   `y = x^(2^level) mod n` together with `clientinitiv alpha omega ot ip`. A third party can implement this. No secrets are involved.
2. **`initivexpand2` (server → client).** The server sends `l` (license chain), `beta`, `omega` (its P-256 identity),
   `proof` (ECDSA signature of `l` with that identity), `ot=1` and `tvd`.
3. **License chain → server ephemeral key.** The client derives a Curve25519 point from the chain:
   `key = ROOT; for block: key = block_pub * clamp(sha512(block[1..])[..32]) + key`.
   `ROOT` (`cd0de2ae…40 8a`) is **hardcoded in the client**. The last block must be `Ephemeral` (type 32), and
   the one before it must be a `Server` block (type 2, or 8 for TS5).
4. **Shared secret.** The client computes `sharedData = derived_key * client_ephemeral_priv`. For the server to derive
   the same AES/MAC keys, it must know the discrete log of `derived_key`, i.e.
   `root_priv + Σ block_priv_i · h_i`. tsproto's `Licenses::derive_private_key(first_key)` makes this explicit:
   it needs *"the root private key"* or the private key of a parent block.
5. Afterwards come `clientek`, `clientinit` and `initserver`, then the channel and client lists. All of this is
   doable once step 4 is solved.

## 3. Can a stock TS3 client connect to a third-party server legitimately?

**No, not with any method found that stays within the rules.**

- **Who issues the key.** `derived_key` always contains the TeamSpeak root point. Only TeamSpeak can issue a
  private key that matches a chain hanging off that root. Real TS3 servers get it from TeamSpeak: the
  built-in default/no-license chain shipped inside the server binary, or a purchased license (`licensekey.dat`).
- **Forging a chain.** You would need `Σ block_pub_i · H(block_i) = K − ROOT` for a point `K` whose private key
  you know. The hash covers the block's own public key, so this is a hash fixed-point or discrete-log problem.
  Even if a weakness existed, exploiting it would mean **bypassing licensing**, which is excluded.
- **The `root=` override.** `initivexpand2 root=` lets a server supply its own root key, but it is a
  **TeaSpeak extension**. Its developer added it so *tsclientlib* could join TeaSpeak (issue #12). TeaSpeak's own
  instructions still require a "TS license" in `protocol_key.txt` for stock clients, so the stock client
  evidently ignores `root=`. This is inferred and was not tested against a stock client.
- **The legacy `initivexpand` handshake.** This is plain P-256 ECDH with no license. The tsclientlib maintainer says
  TeamSpeak's own programs became incompatible with it. TeaSpeak staff say stock clients **3.2.x and newer**
  need a license ("users need to switch to older version"). Pinning users to TS3 clients older than 3.2
  (from around 2018) is not a realistic product.
- **How TeaSpeak did it.** TeaSpeak supported current stock clients only through `protocol_key.txt`, a TeamSpeak
  license chain plus its private key that was handed around on forums. That is exactly the "leaked or extracted
  key" path this project forbids.
- **Buying a license.** Using a purchased TeamSpeak license (for example a commercial or AAL license) to power a
  *third-party* server implementation would still mean extracting its private key from the license file.
  It almost certainly breaches TeamSpeak's license terms. **Unverified:** whether TeamSpeak would grant this
  explicitly. Only TeamSpeak can answer, and the answer would have to be in writing.

## 4. Proof of concept (step 3): not built, deliberately

A stock client cannot complete `initivexpand2` against our server without a key we are not allowed to have.
A PoC could only "work" by:
- using a forged or extracted key, which is forbidden, or
- testing against tsclientlib with a self-generated `root=`, which proves nothing about stock clients.

Nothing ran end to end. Also, no TS3 client and no murmur server are installed on this machine
(`ts3client`, `murmurd` and `mumble-server` were not found), so even the unaffected parts could not be tested
against the real programs yet.

### What remains unverified
- The stock TS3 3.6.x client's handling of `root=` and of legacy `initivexpand`. The conclusions come from
  third-party statements and code, not a packet capture. Confirming would take a stock client, a local TS3 server
  and a capture (allowed, since it only observes).
- Whether TeamSpeak offers any license that explicitly permits third-party server implementations.

## 5. Alternatives

### A. Puppeting bridge with a real TeamSpeak 3 server (recommended)
TS3 users connect to a **real, unmodified TS3 server** (the official binary, run under its own license terms).
The gateway is a **TS3 client** (tsproto: the client-side handshake is legitimate, because the gateway checks the
server's license and does not forge one) plus Mumble clients:

- **TS3 → Mumble:** one Mumble connection per TS3 user, as originally planned (stable cert per TS3 UID,
  mirrored nickname, channel, mute/deaf/away, text, voice).
- **Mumble → TS3:** two options, which need a decision:
  - **A1, full puppets:** one TS3 client connection per Mumble user. This gives the most faithful result.
    It costs TS3 slots (the free "no license" server tier is limited, 32 slots as I recall, *verify*) and
    may trigger TS3 anti-flood or connection limits.
  - **A2, one relay bot per occupied TS3 channel:** it forwards Mumble voice into TS3 (mixed, or
    selected per active speaker) and lists Mumble users in the channel description. It costs fewer slots
    but gives lower fidelity.
- **Receiving voice:** a TS3 client only receives voice for its own channel, so each occupied channel
  needs at least one gateway client in it. The puppets or relay bots cover this naturally.
- **Observing:** ServerQuery (the official admin API) is optional for observing and for channel and permission
  admin.
- **Mumble side:** nothing changes, and the upstream server stays unmodified.
- **Caveat:** tsproto clients send `client_version_sign`, a publicly known signature from an old official client,
  as every TS3 bot does. It is not a private key, but it is a gray area worth acknowledging.

### B. Reusable building blocks
Most of the planned translation layer survives the pivot unchanged: the Mumble backend per session, cert-per-UID
identity, channel and user mapping, text, the Opus jitter buffer and transcoding, config, session cap and shutdown.
Only the frontend changes, from "TS3 server" to "TS3 client(s) on a real TS3 server".

### C. Ask TeamSpeak
Request written permission or a license for a third-party server implementation. This is slow and unlikely,
and it does not block option A.

## 6. Risks for option A
- Slot limits and anti-flood on the TS3 server. This needs config; it does not mean modifying the server.
- Voice latency through two hops, and per-channel voice visibility.
- tsclientlib/tsproto activity is low (latest commit 2026-07). We'd depend on it or vendor parts of it.
- TS3 is in maintenance while TeamSpeak focuses on TS6, so the long-term client base is uncertain.

---

## 7. Spike for the approved architecture (puppeting bridge), 2026-11-03

The user approved alternative A with **one TS3 puppet per Mumble user** (DECISIONS D5). A local spike ran against
the official `teamspeak:3.13` image ("No License", **32 slots**) and `mumblevoip/mumble-server:latest`
(see `dev/compose.dev.yml`).

| Check | Result | How |
|---|---|---|
| tsclientlib (git `ee3bc6f`) connects to the official TS3 server: RSA puzzle, `initivexpand2` license verification, `clientek`, `initserver` | **Verified** | `cargo run --example spike_ts3` |
| Reads channel and client list; connection stays up for 20 s | **Verified** | same |
| Mumble control connection with a **generated client certificate**: Version, Authenticate, ChannelState/UserState, ServerSync, pings for 20 s | **Verified** | `cargo run --example spike_mumble` |
| 12 simultaneous TS3 clients from one IP | **Failed with default settings**: 9 of 12 got `ConnectFailedBanned` (anti-flood IP block; the ban list stays empty, so it is temporary) | `spike_ts3_many 12` |
| Same with `virtualserver_antiflood_points_needed_ip_block=100000`, set through ServerQuery | **Verified: 25 of 25** connected and held 15 s | `spike_ts3_many 25` |

### New findings and risks
- **Anti-flood.** The bundled TS3 server must be configured to tolerate many connections from the gateway's IP.
  The gateway will apply this through ServerQuery at startup (documented, and switchable off). This is a settings
  change on the bundled server, not a modification of it.
- **Slot ceiling.** In the free tier, *TS3 users + Mumble puppets* must stay at or below 32. ServerQuery logins
  do not use slots. This is a hard product limit for non-licensed setups.
- **Mumble crate.** `mumble-protocol-2x` 0.6 is only a codec (control, voice, OCB2 crypt), so the session logic,
  ping, UDP and crypt resync are ours. It works with murmur when the client announces version 1.4 (legacy UDP
  voice format).
- **Still unverified:** voice in either direction, a real stock TS3 client in the loop (not installed here),
  and Mumble registration persisting across reconnects with the same certificate.
