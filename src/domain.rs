use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(u32);

impl Date {
    pub fn from_yyyymmdd(value: u32) -> Result<Self, DomainError> {
        let year = value / 10_000;
        let month = (value / 100) % 100;
        let day = value % 100;
        if year < 1900 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Err(DomainError::InvalidDate(value));
        }
        Ok(Self(value))
    }

    pub const fn yyyymmdd(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Self = Self(0);

    pub const fn from_cents(cents: i64) -> Self {
        Self(cents)
    }

    pub const fn cents(self) -> i64 {
        self.0
    }

    fn checked_add(self, other: Self) -> Result<Self, DomainError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow)
    }

    fn checked_sub(self, other: Self) -> Result<Self, DomainError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatePerHundred(u32);

impl RatePerHundred {
    /// Premium cents charged for each $100 of payroll.
    pub const fn from_cents(cents: u32) -> Self {
        Self(cents)
    }

    pub const fn cents(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exposure {
    pub class_code: String,
    pub payroll: Money,
    pub rate: RatePerHundred,
}

impl Exposure {
    pub fn new(
        class_code: impl Into<String>,
        payroll: Money,
        rate: RatePerHundred,
    ) -> Result<Self, DomainError> {
        let class_code = class_code.into();
        if class_code.trim().is_empty() {
            return Err(DomainError::MissingField("class_code"));
        }
        if payroll.cents() < 0 {
            return Err(DomainError::NegativeAmount("payroll"));
        }
        Ok(Self {
            class_code,
            payroll,
            rate,
        })
    }
}

pub fn calculate_premium(exposures: &[Exposure]) -> Result<Money, DomainError> {
    exposures.iter().try_fold(Money::ZERO, |total, exposure| {
        let product = i128::from(exposure.payroll.cents()) * i128::from(exposure.rate.cents());
        // Payroll is stored in cents. One $100 exposure unit is 10,000 cents.
        let rounded = (product + 5_000) / 10_000;
        let cents = i64::try_from(rounded).map_err(|_| DomainError::ArithmeticOverflow)?;
        total.checked_add(Money::from_cents(cents))
    })
}

fn apply_basis_points(amount: Money, basis_points: u32) -> Result<Money, DomainError> {
    if amount.cents() < 0 {
        return Err(DomainError::NegativeAmount("basis amount"));
    }
    let product = i128::from(amount.cents()) * i128::from(basis_points);
    let rounded = (product + 5_000) / 10_000;
    i64::try_from(rounded)
        .map(Money::from_cents)
        .map_err(|_| DomainError::ArithmeticOverflow)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationStatus {
    Submitted,
    Approved,
    Declined,
}

impl ApplicationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Declined => "declined",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnderwritingDecision {
    pub approved: bool,
    pub decided_on: Date,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub applicant_name: String,
    pub submitted_on: Date,
    pub status: ApplicationStatus,
    pub exposures: Vec<Exposure>,
    pub decision: Option<UnderwritingDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayrollReport {
    pub id: String,
    pub period_start: Date,
    pub period_end: Date,
    pub exposures: Vec<Exposure>,
    pub calculated_premium: Money,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditKind {
    Physical,
    Final,
}

impl AuditKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Physical => "physical",
            Self::Final => "final",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PremiumAudit {
    pub id: String,
    pub kind: AuditKind,
    pub completed_on: Date,
    pub exposures: Vec<Exposure>,
    pub audited_premium: Money,
    pub adjustment: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub id: String,
    pub policy_number: String,
    pub effective_on: Date,
    pub expires_on: Date,
    pub exposures: Vec<Exposure>,
    pub deposit_premium: Money,
    pub payroll_reports: Vec<PayrollReport>,
    pub audits: Vec<PremiumAudit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LossRecord {
    pub id: String,
    pub occurred_on: Date,
    pub paid: Money,
    pub reserved: Money,
}

impl LossRecord {
    pub fn incurred(&self) -> Result<Money, DomainError> {
        self.paid.checked_add(self.reserved)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub id: String,
    pub holder_name: String,
    pub issued_on: Date,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commission {
    pub id: String,
    pub recipient_name: String,
    pub basis_points: u32,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiaryEntry {
    pub id: String,
    pub due_on: Date,
    pub note: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalProjection {
    pub id: String,
    pub projected_exposures: Vec<Exposure>,
    pub experience_modifier_basis_points: u32,
    pub projected_premium: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsuranceCase {
    pub id: String,
    pub application: Application,
    pub policy: Option<Policy>,
    pub losses: Vec<LossRecord>,
    pub certificates: Vec<Certificate>,
    pub commissions: Vec<Commission>,
    pub diary: Vec<DiaryEntry>,
    pub renewal: Option<RenewalProjection>,
}

impl InsuranceCase {
    pub fn new(
        id: impl Into<String>,
        application_id: impl Into<String>,
        applicant_name: impl Into<String>,
        submitted_on: Date,
        exposures: Vec<Exposure>,
    ) -> Result<Self, DomainError> {
        let id = id.into();
        let applicant_name = applicant_name.into();
        if id.trim().is_empty() {
            return Err(DomainError::MissingField("case id"));
        }
        if applicant_name.trim().is_empty() {
            return Err(DomainError::MissingField("applicant_name"));
        }
        if exposures.is_empty() {
            return Err(DomainError::MissingExposure);
        }
        Ok(Self {
            id,
            application: Application {
                id: application_id.into(),
                applicant_name,
                submitted_on,
                status: ApplicationStatus::Submitted,
                exposures,
                decision: None,
            },
            policy: None,
            losses: Vec::new(),
            certificates: Vec::new(),
            commissions: Vec::new(),
            diary: Vec::new(),
            renewal: None,
        })
    }

    pub fn decide(
        &mut self,
        approved: bool,
        decided_on: Date,
        rationale: impl Into<String>,
    ) -> Result<(), DomainError> {
        if self.application.status != ApplicationStatus::Submitted {
            return Err(DomainError::InvalidTransition(
                "application already decided",
            ));
        }
        self.application.status = if approved {
            ApplicationStatus::Approved
        } else {
            ApplicationStatus::Declined
        };
        self.application.decision = Some(UnderwritingDecision {
            approved,
            decided_on,
            rationale: rationale.into(),
        });
        Ok(())
    }

    pub fn bind_policy(
        &mut self,
        id: impl Into<String>,
        policy_number: impl Into<String>,
        effective_on: Date,
        expires_on: Date,
    ) -> Result<(), DomainError> {
        if self.application.status != ApplicationStatus::Approved {
            return Err(DomainError::InvalidTransition(
                "only approved applications can bind",
            ));
        }
        if effective_on >= expires_on {
            return Err(DomainError::InvalidDateRange);
        }
        let exposures = self.application.exposures.clone();
        let deposit_premium = calculate_premium(&exposures)?;
        self.policy = Some(Policy {
            id: id.into(),
            policy_number: policy_number.into(),
            effective_on,
            expires_on,
            exposures,
            deposit_premium,
            payroll_reports: Vec::new(),
            audits: Vec::new(),
        });
        Ok(())
    }

    pub fn record_payroll(
        &mut self,
        id: impl Into<String>,
        period_start: Date,
        period_end: Date,
        exposures: Vec<Exposure>,
    ) -> Result<Money, DomainError> {
        if period_start > period_end {
            return Err(DomainError::InvalidDateRange);
        }
        let premium = calculate_premium(&exposures)?;
        self.policy_mut()?.payroll_reports.push(PayrollReport {
            id: id.into(),
            period_start,
            period_end,
            exposures,
            calculated_premium: premium,
        });
        Ok(premium)
    }

    pub fn complete_audit(
        &mut self,
        id: impl Into<String>,
        kind: AuditKind,
        completed_on: Date,
        exposures: Vec<Exposure>,
    ) -> Result<Money, DomainError> {
        let audited_premium = calculate_premium(&exposures)?;
        let policy = self.policy_mut()?;
        let adjustment = audited_premium.checked_sub(policy.deposit_premium)?;
        policy.audits.push(PremiumAudit {
            id: id.into(),
            kind,
            completed_on,
            exposures,
            audited_premium,
            adjustment,
        });
        Ok(adjustment)
    }

    pub fn record_loss(&mut self, loss: LossRecord) -> Result<(), DomainError> {
        if loss.paid.cents() < 0 || loss.reserved.cents() < 0 {
            return Err(DomainError::NegativeAmount("loss"));
        }
        self.policy_ref()?;
        self.losses.push(loss);
        Ok(())
    }

    pub fn issue_certificate(&mut self, certificate: Certificate) -> Result<(), DomainError> {
        self.policy_ref()?;
        if certificate.holder_name.trim().is_empty() {
            return Err(DomainError::MissingField("certificate holder"));
        }
        self.certificates.push(certificate);
        Ok(())
    }

    pub fn calculate_commission(
        &mut self,
        id: impl Into<String>,
        recipient_name: impl Into<String>,
        basis_points: u32,
    ) -> Result<Money, DomainError> {
        let amount = apply_basis_points(self.policy_ref()?.deposit_premium, basis_points)?;
        self.commissions.push(Commission {
            id: id.into(),
            recipient_name: recipient_name.into(),
            basis_points,
            amount,
        });
        Ok(amount)
    }

    pub fn add_diary_entry(&mut self, entry: DiaryEntry) -> Result<(), DomainError> {
        if entry.note.trim().is_empty() {
            return Err(DomainError::MissingField("diary note"));
        }
        self.diary.push(entry);
        Ok(())
    }

    pub fn project_renewal(
        &mut self,
        id: impl Into<String>,
        projected_exposures: Vec<Exposure>,
        experience_modifier_basis_points: u32,
    ) -> Result<Money, DomainError> {
        self.policy_ref()?;
        let manual_premium = calculate_premium(&projected_exposures)?;
        let projected_premium =
            apply_basis_points(manual_premium, experience_modifier_basis_points)?;
        self.renewal = Some(RenewalProjection {
            id: id.into(),
            projected_exposures,
            experience_modifier_basis_points,
            projected_premium,
        });
        Ok(projected_premium)
    }

    pub fn loss_summary(&self) -> Result<Money, DomainError> {
        self.losses.iter().try_fold(Money::ZERO, |total, loss| {
            total.checked_add(loss.incurred()?)
        })
    }

    fn policy_ref(&self) -> Result<&Policy, DomainError> {
        self.policy.as_ref().ok_or(DomainError::PolicyNotBound)
    }

    fn policy_mut(&mut self) -> Result<&mut Policy, DomainError> {
        self.policy.as_mut().ok_or(DomainError::PolicyNotBound)
    }
}

pub fn synthetic_case() -> Result<InsuranceCase, DomainError> {
    let initial = vec![Exposure::new(
        "OPS-100",
        Money::from_cents(125_000_000),
        RatePerHundred::from_cents(185),
    )?];
    let mut case = InsuranceCase::new(
        "CASE-DEMO-1",
        "APP-DEMO-1",
        "Synthetic Fabrication Cooperative",
        Date::from_yyyymmdd(20260105)?,
        initial,
    )?;
    case.decide(
        true,
        Date::from_yyyymmdd(20260108)?,
        "Synthetic controls satisfy the configured appetite",
    )?;
    case.bind_policy(
        "POL-DEMO-1",
        "RH-DEMO-2026",
        Date::from_yyyymmdd(20260201)?,
        Date::from_yyyymmdd(20270201)?,
    )?;
    case.record_payroll(
        "PAY-DEMO-1",
        Date::from_yyyymmdd(20260201)?,
        Date::from_yyyymmdd(20260430)?,
        vec![Exposure::new(
            "OPS-100",
            Money::from_cents(32_500_000),
            RatePerHundred::from_cents(185),
        )?],
    )?;
    case.complete_audit(
        "AUD-DEMO-1",
        AuditKind::Final,
        Date::from_yyyymmdd(20270210)?,
        vec![Exposure::new(
            "OPS-100",
            Money::from_cents(131_000_000),
            RatePerHundred::from_cents(185),
        )?],
    )?;
    case.record_loss(LossRecord {
        id: "LOSS-DEMO-1".into(),
        occurred_on: Date::from_yyyymmdd(20260714)?,
        paid: Money::from_cents(125_000),
        reserved: Money::from_cents(75_000),
    })?;
    case.issue_certificate(Certificate {
        id: "CERT-DEMO-1".into(),
        holder_name: "Synthetic Property Partners".into(),
        issued_on: Date::from_yyyymmdd(20260202)?,
    })?;
    case.calculate_commission("COMM-DEMO-1", "Synthetic Agency", 750)?;
    case.add_diary_entry(DiaryEntry {
        id: "DIARY-DEMO-1".into(),
        due_on: Date::from_yyyymmdd(20261201)?,
        note: "Request projected payroll for renewal".into(),
        completed: false,
    })?;
    case.project_renewal(
        "RENEW-DEMO-1",
        vec![Exposure::new(
            "OPS-100",
            Money::from_cents(138_000_000),
            RatePerHundred::from_cents(190),
        )?],
        10_250,
    )?;
    Ok(case)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    ArithmeticOverflow,
    InvalidDate(u32),
    InvalidDateRange,
    InvalidTransition(&'static str),
    MissingExposure,
    MissingField(&'static str),
    NegativeAmount(&'static str),
    PolicyNotBound,
}

impl Display for DomainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArithmeticOverflow => write!(formatter, "arithmetic overflow"),
            Self::InvalidDate(value) => write!(formatter, "invalid YYYYMMDD date: {value}"),
            Self::InvalidDateRange => write!(formatter, "invalid date range"),
            Self::InvalidTransition(message) => write!(formatter, "invalid transition: {message}"),
            Self::MissingExposure => write!(formatter, "at least one exposure is required"),
            Self::MissingField(field) => write!(formatter, "missing required field: {field}"),
            Self::NegativeAmount(field) => write!(formatter, "negative amount is invalid: {field}"),
            Self::PolicyNotBound => write!(formatter, "policy is not bound"),
        }
    }
}

impl Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: u32) -> Date {
        Date::from_yyyymmdd(value).unwrap()
    }

    fn exposure(payroll_cents: i64, rate_cents: u32) -> Exposure {
        Exposure::new(
            "TEST-CLASS",
            Money::from_cents(payroll_cents),
            RatePerHundred::from_cents(rate_cents),
        )
        .unwrap()
    }

    #[test]
    fn premium_uses_integer_cents_and_rounds_half_up() {
        let premium = calculate_premium(&[exposure(12_345_678, 185)]).unwrap();
        assert_eq!(premium, Money::from_cents(228_395));
    }

    #[test]
    fn declined_application_cannot_bind() {
        let mut case = InsuranceCase::new(
            "CASE-1",
            "APP-1",
            "Synthetic Applicant",
            date(20260101),
            vec![exposure(1_000_000, 100)],
        )
        .unwrap();
        case.decide(false, date(20260102), "Outside configured appetite")
            .unwrap();
        assert!(matches!(
            case.bind_policy("POL-1", "TEST-1", date(20260201), date(20270201)),
            Err(DomainError::InvalidTransition(_))
        ));
    }

    #[test]
    fn audit_adjustment_is_audited_less_deposit() {
        let mut case = InsuranceCase::new(
            "CASE-1",
            "APP-1",
            "Synthetic Applicant",
            date(20260101),
            vec![exposure(10_000_000, 200)],
        )
        .unwrap();
        case.decide(true, date(20260102), "Approved").unwrap();
        case.bind_policy("POL-1", "TEST-1", date(20260201), date(20270201))
            .unwrap();
        let adjustment = case
            .complete_audit(
                "AUD-1",
                AuditKind::Final,
                date(20270202),
                vec![exposure(12_000_000, 200)],
            )
            .unwrap();
        assert_eq!(adjustment, Money::from_cents(40_000));
    }

    #[test]
    fn loss_summary_includes_paid_and_reserved() {
        let case = synthetic_case().unwrap();
        assert_eq!(case.loss_summary().unwrap(), Money::from_cents(200_000));
    }

    #[test]
    fn commission_and_renewal_are_integer_safe() {
        let case = synthetic_case().unwrap();
        assert_eq!(case.commissions[0].amount, Money::from_cents(173_438));
        assert_eq!(
            case.renewal.unwrap().projected_premium,
            Money::from_cents(2_687_550)
        );
    }
}
