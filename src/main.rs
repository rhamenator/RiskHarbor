use risk_harbor::{PostgresStore, synthetic_case};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let insurance_case = synthetic_case()?;
    let policy = insurance_case
        .policy
        .as_ref()
        .expect("demo policy is bound");
    println!(
        "case={} status={} deposit_premium_cents={} incurred_losses_cents={} renewal_premium_cents={}",
        insurance_case.id,
        insurance_case.application.status.as_str(),
        policy.deposit_premium.cents(),
        insurance_case.loss_summary()?.cents(),
        insurance_case
            .renewal
            .as_ref()
            .expect("demo renewal exists")
            .projected_premium
            .cents()
    );

    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        let mut store = PostgresStore::connect(&database_url).await?;
        store.migrate().await?;
        store.save_case(&insurance_case).await?;
        println!("persisted_to_postgresql=true");
    } else {
        println!("persisted_to_postgresql=false (set DATABASE_URL to enable)");
    }
    Ok(())
}
