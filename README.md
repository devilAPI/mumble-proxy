# mumble-proxy
A proxy allowing TS§ clients to connect to a mumble server. Work in progress.

## Status: on hold (2026-11-03)
Phase 0 (feasibility) is done:
- A direct TS3-server frontend is a **no-go**, because stock TS3 clients require a TeamSpeak-issued license key. See [FEASIBILITY.md](FEASIBILITY.md).
- The approved design is a puppeting bridge with a bundled official TS3 server; the plan is in [PLAN.md](PLAN.md).
- The decisions so far are logged in [DECISIONS.md](DECISIONS.md).
- The spikes in `examples/` show that both protocol sides work against the real servers.

Phase 1 has not started. The three open default decisions are listed at the end of FEASIBILITY.md §7 and in the PLAN.
