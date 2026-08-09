# RiskHarbor

RiskHarbor is a clean-room insurance-operations core with integer-safe premium
calculations and PostgreSQL persistence. Its first vertical slice follows a
synthetic account through intake, underwriting, policy binding, payroll
reporting, premium audit, loss reporting, certificate issuance, commission,
diary follow-up, and renewal projection.

## Run the deterministic demonstration

```powershell
cargo run
```

To exercise PostgreSQL persistence:

```powershell
docker compose up -d
$env:DATABASE_URL = "postgresql://riskharbor:riskharbor@localhost:54329/riskharbor"
cargo test
cargo run
```

Money is stored as integer cents. Exposure rates are stored as premium cents
per $100 of payroll. Experience modifiers and commissions use integer basis
points. No floating-point arithmetic is used for financial results.

## Current boundary

- Application intake and explicit underwriting decisions
- Policy terms, classifications, and payroll exposures
- Deposit, reported, and audited premium calculations
- Paid and reserved loss summaries
- Certificates, commissions, diary work, and renewal projections
- Transactional PostgreSQL snapshot persistence and append-only case events

Regulatory transmission is deliberately outside this initial slice. Any future
adapter must be built from current authoritative specifications and configured
per jurisdiction.

See [Architecture](docs/ARCHITECTURE.md) and
[Clean-room boundary](docs/CLEAN_ROOM_BOUNDARY.md).
