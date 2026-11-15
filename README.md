# flags-2-env-sync

Wraps github.com/opto-sync for flags-2-env with SQLite on clients and Postgres/Supabase on servers.

`decide_envelope` is the pure, exhaustive synchronization admission core. A
typed `SyncMode` excludes Boolean-state ambiguity, `PayloadLimit` excludes a
zero bound, and effects stay in `SyncBoundary`. Unit tests enumerate the finite
boundary table; Kani proves disabled-mode rejection and the arbitrary-bound
payload safety properties in `.github/workflows/formal-methods.yml`.

`decide_incremental` extends the same admission policy to a counted stream of
chunks. It rejects `usize` overflow and cumulative payloads beyond the bound
before a caller performs any storage or network effect. The 288-case test grid,
exact-boundary cases, and maximum-integer regressions are host-side Rust tests.
A successful decision is not a guarantee that downstream I/O succeeded.

These proofs cover admission decisions only. They do not prove network
delivery, storage durability, remote authorization, or opto-sync behavior.
