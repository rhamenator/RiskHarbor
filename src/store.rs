use crate::{InsuranceCase, Money};
use tokio::task::JoinHandle;
use tokio_postgres::{Client, NoTls};

pub struct PostgresStore {
    client: Client,
    connection_task: JoinHandle<()>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseSummary {
    pub deposit_premium: Money,
    pub incurred_losses: Money,
    pub certificate_count: i64,
    pub open_diary_count: i64,
    pub projected_renewal_premium: Option<Money>,
}

impl PostgresStore {
    pub async fn connect(connection_string: &str) -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
        let connection_task = tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("PostgreSQL connection error: {error}");
            }
        });
        Ok(Self {
            client,
            connection_task,
        })
    }

    pub async fn migrate(&self) -> Result<(), tokio_postgres::Error> {
        self.client
            .batch_execute(include_str!("../migrations/0001_initial.sql"))
            .await
    }

    pub async fn save_case(
        &mut self,
        insurance_case: &InsuranceCase,
    ) -> Result<(), tokio_postgres::Error> {
        let transaction = self.client.transaction().await?;
        transaction
            .execute(
                "INSERT INTO insurance_cases (id) VALUES ($1) ON CONFLICT (id) DO NOTHING",
                &[&insurance_case.id],
            )
            .await?;
        transaction
            .execute(
                "INSERT INTO applications (id, case_id, applicant_name, submitted_on, status) \
                 VALUES ($1,$2,$3,$4,$5) ON CONFLICT (id) DO UPDATE SET \
                 applicant_name=EXCLUDED.applicant_name, status=EXCLUDED.status",
                &[
                    &insurance_case.application.id,
                    &insurance_case.id,
                    &insurance_case.application.applicant_name,
                    &(insurance_case.application.submitted_on.yyyymmdd() as i32),
                    &insurance_case.application.status.as_str(),
                ],
            )
            .await?;
        transaction
            .execute(
                "DELETE FROM application_exposures WHERE application_id=$1",
                &[&insurance_case.application.id],
            )
            .await?;
        for exposure in &insurance_case.application.exposures {
            transaction
                .execute(
                    "INSERT INTO application_exposures \
                     (application_id,class_code,payroll_cents,rate_cents_per_hundred) \
                     VALUES ($1,$2,$3,$4)",
                    &[
                        &insurance_case.application.id,
                        &exposure.class_code,
                        &exposure.payroll.cents(),
                        &(exposure.rate.cents() as i32),
                    ],
                )
                .await?;
        }
        if let Some(decision) = &insurance_case.application.decision {
            transaction
                .execute(
                    "INSERT INTO underwriting_decisions \
                     (application_id,approved,decided_on,rationale) VALUES ($1,$2,$3,$4) \
                     ON CONFLICT (application_id) DO UPDATE SET approved=EXCLUDED.approved, \
                     decided_on=EXCLUDED.decided_on, rationale=EXCLUDED.rationale",
                    &[
                        &insurance_case.application.id,
                        &decision.approved,
                        &(decision.decided_on.yyyymmdd() as i32),
                        &decision.rationale,
                    ],
                )
                .await?;
        }
        if let Some(policy) = &insurance_case.policy {
            transaction
                .execute(
                    "INSERT INTO policies \
                     (id,case_id,policy_number,effective_on,expires_on,deposit_premium_cents) \
                     VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO UPDATE SET \
                     policy_number=EXCLUDED.policy_number, deposit_premium_cents=EXCLUDED.deposit_premium_cents",
                    &[
                        &policy.id,
                        &insurance_case.id,
                        &policy.policy_number,
                        &(policy.effective_on.yyyymmdd() as i32),
                        &(policy.expires_on.yyyymmdd() as i32),
                        &policy.deposit_premium.cents(),
                    ],
                )
                .await?;
            transaction
                .execute(
                    "DELETE FROM policy_exposures WHERE policy_id=$1",
                    &[&policy.id],
                )
                .await?;
            for exposure in &policy.exposures {
                transaction
                    .execute(
                        "INSERT INTO policy_exposures \
                         (policy_id,class_code,payroll_cents,rate_cents_per_hundred) VALUES ($1,$2,$3,$4)",
                        &[
                            &policy.id,
                            &exposure.class_code,
                            &exposure.payroll.cents(),
                            &(exposure.rate.cents() as i32),
                        ],
                    )
                    .await?;
            }
            for report in &policy.payroll_reports {
                transaction
                    .execute(
                        "INSERT INTO payroll_reports \
                         (id,policy_id,period_start,period_end,calculated_premium_cents) \
                         VALUES ($1,$2,$3,$4,$5) ON CONFLICT (id) DO UPDATE SET \
                         calculated_premium_cents=EXCLUDED.calculated_premium_cents",
                        &[
                            &report.id,
                            &policy.id,
                            &(report.period_start.yyyymmdd() as i32),
                            &(report.period_end.yyyymmdd() as i32),
                            &report.calculated_premium.cents(),
                        ],
                    )
                    .await?;
                transaction
                    .execute(
                        "DELETE FROM payroll_exposures WHERE report_id=$1",
                        &[&report.id],
                    )
                    .await?;
                for exposure in &report.exposures {
                    transaction
                        .execute(
                            "INSERT INTO payroll_exposures \
                             (report_id,class_code,payroll_cents,rate_cents_per_hundred) VALUES ($1,$2,$3,$4)",
                            &[
                                &report.id,
                                &exposure.class_code,
                                &exposure.payroll.cents(),
                                &(exposure.rate.cents() as i32),
                            ],
                        )
                        .await?;
                }
            }
            for audit in &policy.audits {
                transaction
                    .execute(
                        "INSERT INTO premium_audits \
                         (id,policy_id,kind,completed_on,audited_premium_cents,adjustment_cents) \
                         VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (id) DO UPDATE SET \
                         audited_premium_cents=EXCLUDED.audited_premium_cents, adjustment_cents=EXCLUDED.adjustment_cents",
                        &[
                            &audit.id,
                            &policy.id,
                            &audit.kind.as_str(),
                            &(audit.completed_on.yyyymmdd() as i32),
                            &audit.audited_premium.cents(),
                            &audit.adjustment.cents(),
                        ],
                    )
                    .await?;
                transaction
                    .execute(
                        "DELETE FROM audit_exposures WHERE audit_id=$1",
                        &[&audit.id],
                    )
                    .await?;
                for exposure in &audit.exposures {
                    transaction
                        .execute(
                            "INSERT INTO audit_exposures \
                             (audit_id,class_code,payroll_cents,rate_cents_per_hundred) VALUES ($1,$2,$3,$4)",
                            &[
                                &audit.id,
                                &exposure.class_code,
                                &exposure.payroll.cents(),
                                &(exposure.rate.cents() as i32),
                            ],
                        )
                        .await?;
                }
            }
        }
        for loss in &insurance_case.losses {
            transaction
                .execute(
                    "INSERT INTO loss_records (id,case_id,occurred_on,paid_cents,reserved_cents) \
                     VALUES ($1,$2,$3,$4,$5) ON CONFLICT (id) DO UPDATE SET \
                     paid_cents=EXCLUDED.paid_cents,reserved_cents=EXCLUDED.reserved_cents",
                    &[
                        &loss.id,
                        &insurance_case.id,
                        &(loss.occurred_on.yyyymmdd() as i32),
                        &loss.paid.cents(),
                        &loss.reserved.cents(),
                    ],
                )
                .await?;
        }
        for certificate in &insurance_case.certificates {
            transaction
                .execute(
                    "INSERT INTO certificates (id,case_id,holder_name,issued_on) VALUES ($1,$2,$3,$4) \
                     ON CONFLICT (id) DO UPDATE SET holder_name=EXCLUDED.holder_name",
                    &[
                        &certificate.id,
                        &insurance_case.id,
                        &certificate.holder_name,
                        &(certificate.issued_on.yyyymmdd() as i32),
                    ],
                )
                .await?;
        }
        for commission in &insurance_case.commissions {
            transaction
                .execute(
                    "INSERT INTO commissions (id,case_id,recipient_name,basis_points,amount_cents) \
                     VALUES ($1,$2,$3,$4,$5) ON CONFLICT (id) DO UPDATE SET amount_cents=EXCLUDED.amount_cents",
                    &[
                        &commission.id,
                        &insurance_case.id,
                        &commission.recipient_name,
                        &(commission.basis_points as i32),
                        &commission.amount.cents(),
                    ],
                )
                .await?;
        }
        for entry in &insurance_case.diary {
            transaction
                .execute(
                    "INSERT INTO diary_entries (id,case_id,due_on,note,completed) VALUES ($1,$2,$3,$4,$5) \
                     ON CONFLICT (id) DO UPDATE SET note=EXCLUDED.note,completed=EXCLUDED.completed",
                    &[
                        &entry.id,
                        &insurance_case.id,
                        &(entry.due_on.yyyymmdd() as i32),
                        &entry.note,
                        &entry.completed,
                    ],
                )
                .await?;
        }
        if let Some(renewal) = &insurance_case.renewal {
            transaction
                .execute(
                    "INSERT INTO renewal_projections \
                     (id,case_id,experience_modifier_basis_points,projected_premium_cents) \
                     VALUES ($1,$2,$3,$4) ON CONFLICT (id) DO UPDATE SET \
                     experience_modifier_basis_points=EXCLUDED.experience_modifier_basis_points, \
                     projected_premium_cents=EXCLUDED.projected_premium_cents",
                    &[
                        &renewal.id,
                        &insurance_case.id,
                        &(renewal.experience_modifier_basis_points as i32),
                        &renewal.projected_premium.cents(),
                    ],
                )
                .await?;
            transaction
                .execute(
                    "DELETE FROM renewal_exposures WHERE renewal_id=$1",
                    &[&renewal.id],
                )
                .await?;
            for exposure in &renewal.projected_exposures {
                transaction
                    .execute(
                        "INSERT INTO renewal_exposures \
                         (renewal_id,class_code,payroll_cents,rate_cents_per_hundred) VALUES ($1,$2,$3,$4)",
                        &[
                            &renewal.id,
                            &exposure.class_code,
                            &exposure.payroll.cents(),
                            &(exposure.rate.cents() as i32),
                        ],
                    )
                    .await?;
            }
        }
        transaction
            .execute(
                "INSERT INTO case_events (case_id,event_type,details) VALUES ($1,'snapshot_saved','synthetic lifecycle persisted')",
                &[&insurance_case.id],
            )
            .await?;
        transaction.commit().await
    }

    pub async fn case_summary(
        &self,
        case_id: &str,
    ) -> Result<Option<CaseSummary>, tokio_postgres::Error> {
        let row = self
            .client
            .query_opt(
                "SELECT p.deposit_premium_cents, \
                 COALESCE((SELECT SUM(l.paid_cents+l.reserved_cents) FROM loss_records l WHERE l.case_id=c.id),0)::BIGINT, \
                 (SELECT COUNT(*) FROM certificates x WHERE x.case_id=c.id), \
                 (SELECT COUNT(*) FROM diary_entries d WHERE d.case_id=c.id AND NOT d.completed), \
                 (SELECT r.projected_premium_cents FROM renewal_projections r WHERE r.case_id=c.id LIMIT 1) \
                 FROM insurance_cases c JOIN policies p ON p.case_id=c.id WHERE c.id=$1",
                &[&case_id],
            )
            .await?;
        Ok(row.map(|row| CaseSummary {
            deposit_premium: Money::from_cents(row.get(0)),
            incurred_losses: Money::from_cents(row.get(1)),
            certificate_count: row.get(2),
            open_diary_count: row.get(3),
            projected_renewal_premium: row.get::<_, Option<i64>>(4).map(Money::from_cents),
        }))
    }
}

impl Drop for PostgresStore {
    fn drop(&mut self) {
        self.connection_task.abort();
    }
}
