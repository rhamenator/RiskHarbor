use risk_harbor::{Money, PostgresStore, synthetic_case};

#[tokio::test]
async fn persists_and_summarizes_complete_lifecycle() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("DATABASE_URL is unset; PostgreSQL integration test skipped");
        return;
    };
    let insurance_case = synthetic_case().unwrap();
    let mut store = PostgresStore::connect(&database_url).await.unwrap();
    store.migrate().await.unwrap();
    store.save_case(&insurance_case).await.unwrap();
    let summary = store
        .case_summary(&insurance_case.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(summary.deposit_premium, Money::from_cents(2_312_500));
    assert_eq!(summary.incurred_losses, Money::from_cents(200_000));
    assert_eq!(summary.certificate_count, 1);
    assert_eq!(summary.open_diary_count, 1);
    assert_eq!(
        summary.projected_renewal_premium,
        Some(Money::from_cents(2_687_550))
    );
}
