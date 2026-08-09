CREATE TABLE IF NOT EXISTS insurance_cases (
    id TEXT PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS applications (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL UNIQUE REFERENCES insurance_cases(id) ON DELETE CASCADE,
    applicant_name TEXT NOT NULL,
    submitted_on INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('submitted','approved','declined'))
);

CREATE TABLE IF NOT EXISTS application_exposures (
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    class_code TEXT NOT NULL,
    payroll_cents BIGINT NOT NULL CHECK (payroll_cents >= 0),
    rate_cents_per_hundred INTEGER NOT NULL CHECK (rate_cents_per_hundred >= 0),
    PRIMARY KEY (application_id,class_code)
);

CREATE TABLE IF NOT EXISTS underwriting_decisions (
    application_id TEXT PRIMARY KEY REFERENCES applications(id) ON DELETE CASCADE,
    approved BOOLEAN NOT NULL,
    decided_on INTEGER NOT NULL,
    rationale TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL UNIQUE REFERENCES insurance_cases(id) ON DELETE CASCADE,
    policy_number TEXT NOT NULL UNIQUE,
    effective_on INTEGER NOT NULL,
    expires_on INTEGER NOT NULL,
    deposit_premium_cents BIGINT NOT NULL CHECK (deposit_premium_cents >= 0),
    CHECK (effective_on < expires_on)
);

CREATE TABLE IF NOT EXISTS policy_exposures (
    policy_id TEXT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    class_code TEXT NOT NULL,
    payroll_cents BIGINT NOT NULL CHECK (payroll_cents >= 0),
    rate_cents_per_hundred INTEGER NOT NULL CHECK (rate_cents_per_hundred >= 0),
    PRIMARY KEY (policy_id,class_code)
);

CREATE TABLE IF NOT EXISTS payroll_reports (
    id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    period_start INTEGER NOT NULL,
    period_end INTEGER NOT NULL,
    calculated_premium_cents BIGINT NOT NULL CHECK (calculated_premium_cents >= 0),
    CHECK (period_start <= period_end)
);

CREATE TABLE IF NOT EXISTS payroll_exposures (
    report_id TEXT NOT NULL REFERENCES payroll_reports(id) ON DELETE CASCADE,
    class_code TEXT NOT NULL,
    payroll_cents BIGINT NOT NULL CHECK (payroll_cents >= 0),
    rate_cents_per_hundred INTEGER NOT NULL CHECK (rate_cents_per_hundred >= 0),
    PRIMARY KEY (report_id,class_code)
);

CREATE TABLE IF NOT EXISTS premium_audits (
    id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('physical','final')),
    completed_on INTEGER NOT NULL,
    audited_premium_cents BIGINT NOT NULL CHECK (audited_premium_cents >= 0),
    adjustment_cents BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_exposures (
    audit_id TEXT NOT NULL REFERENCES premium_audits(id) ON DELETE CASCADE,
    class_code TEXT NOT NULL,
    payroll_cents BIGINT NOT NULL CHECK (payroll_cents >= 0),
    rate_cents_per_hundred INTEGER NOT NULL CHECK (rate_cents_per_hundred >= 0),
    PRIMARY KEY (audit_id,class_code)
);

CREATE TABLE IF NOT EXISTS loss_records (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES insurance_cases(id) ON DELETE CASCADE,
    occurred_on INTEGER NOT NULL,
    paid_cents BIGINT NOT NULL CHECK (paid_cents >= 0),
    reserved_cents BIGINT NOT NULL CHECK (reserved_cents >= 0)
);

CREATE TABLE IF NOT EXISTS certificates (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES insurance_cases(id) ON DELETE CASCADE,
    holder_name TEXT NOT NULL,
    issued_on INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS commissions (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES insurance_cases(id) ON DELETE CASCADE,
    recipient_name TEXT NOT NULL,
    basis_points INTEGER NOT NULL CHECK (basis_points >= 0),
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0)
);

CREATE TABLE IF NOT EXISTS diary_entries (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES insurance_cases(id) ON DELETE CASCADE,
    due_on INTEGER NOT NULL,
    note TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS renewal_projections (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL UNIQUE REFERENCES insurance_cases(id) ON DELETE CASCADE,
    experience_modifier_basis_points INTEGER NOT NULL CHECK (experience_modifier_basis_points >= 0),
    projected_premium_cents BIGINT NOT NULL CHECK (projected_premium_cents >= 0)
);

CREATE TABLE IF NOT EXISTS renewal_exposures (
    renewal_id TEXT NOT NULL REFERENCES renewal_projections(id) ON DELETE CASCADE,
    class_code TEXT NOT NULL,
    payroll_cents BIGINT NOT NULL CHECK (payroll_cents >= 0),
    rate_cents_per_hundred INTEGER NOT NULL CHECK (rate_cents_per_hundred >= 0),
    PRIMARY KEY (renewal_id,class_code)
);

CREATE TABLE IF NOT EXISTS case_events (
    sequence BIGSERIAL PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES insurance_cases(id) ON DELETE CASCADE,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    event_type TEXT NOT NULL,
    details TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_loss_records_case ON loss_records(case_id);
CREATE INDEX IF NOT EXISTS idx_diary_open_due ON diary_entries(due_on) WHERE NOT completed;
CREATE INDEX IF NOT EXISTS idx_case_events_case_sequence ON case_events(case_id,sequence);
