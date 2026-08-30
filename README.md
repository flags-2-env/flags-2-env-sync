# flags-2-env-sync

Wraps github.com/opto-sync for flags-2-env with SQLite on clients and Postgres/Supabase on servers.

`decide_envelope` is the pure, exhaustive synchronization admission core. A
typed `SyncMode` excludes Boolean-state ambiguity, `PayloadLimit` excludes a
zero bound, and effects stay in `SyncBoundary`. Unit tests enumerate the finite
boundary table; Kani proves disabled-mode rejection and the arbitrary-bound
payload safety properties in `.github/workflows/formal-methods.yml`.

These proofs cover admission decisions only. They do not prove network
delivery, storage durability, remote authorization, or opto-sync behavior.
