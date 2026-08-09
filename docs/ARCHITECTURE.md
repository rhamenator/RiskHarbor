# Architecture

The domain module owns lifecycle rules and integer-safe calculations without a
database dependency. `PostgresStore` persists a complete case inside one
transaction and exposes a reporting summary used by the integration test.

The initial aggregate is intentionally narrow: one application may produce one
policy, with subordinate exposures, payroll reports, premium audits, losses,
certificates, commissions, diary entries, and one renewal projection. The
schema keeps those records separate so later APIs can add independent commands,
optimistic concurrency, and role-based approvals without changing the financial
formulas.

The `case_events` table is append-only operational evidence. It is not yet a
full event-sourcing implementation.
