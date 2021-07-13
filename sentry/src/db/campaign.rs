use crate::db::{DbPool, PoolError};
use primitives::{Campaign, CampaignId, ChannelId};
use tokio_postgres::types::Json;

pub async fn insert_campaign(pool: &DbPool, campaign: &Campaign) -> Result<bool, PoolError> {
    let client = pool.get().await?;
    let ad_units = Json(campaign.ad_units.clone());
    let stmt = client.prepare("INSERT INTO campaigns (id, channel_id, channel, creator, budget, validators, title, pricing_bounds, event_submission, ad_units, targeting_rules, created, active_from, active_to) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)").await?;
    let inserted = client
        .execute(
            &stmt,
            &[
                &campaign.id,
                &campaign.channel.id(),
                &campaign.channel,
                &campaign.creator,
                &campaign.budget,
                &campaign.validators,
                &campaign.title,
                &campaign.pricing_bounds,
                &campaign.event_submission,
                &ad_units,
                &campaign.targeting_rules,
                &campaign.created,
                &campaign.active.from,
                &campaign.active.to,
            ],
        )
        .await?;

    let inserted = inserted == 1;
    Ok(inserted)
}

/// ```text
/// SELECT id, channel, creator, budget, validators, title, pricing_bounds, event_submission, ad_units, targeting_rules, created, active_from, active_to FROM campaigns
/// WHERE id = $1
/// ```
pub async fn fetch_campaign(
    pool: DbPool,
    campaign: &CampaignId,
) -> Result<Option<Campaign>, PoolError> {
    let client = pool.get().await?;
    let statement = client.prepare("SELECT id, channel, creator, budget, validators, title, pricing_bounds, event_submission, ad_units, targeting_rules, created, active_from, active_to FROM campaigns WHERE id = $1").await?;

    let row = client.query_opt(&statement, &[&campaign]).await?;

    Ok(row.as_ref().map(Campaign::from))
}

// TODO: We might need to use LIMIT to implement pagination
pub async fn get_campaigns_by_channel(
    pool: &DbPool,
    channel_id: &ChannelId,
) -> Result<Vec<Campaign>, PoolError> {
    let client = pool.get().await?;
    let statement = client.prepare("SELECT id, channel, creator, budget, validators, title, pricing_bounds, event_submission, ad_units, targeting_rules, created, active_from, active_to FROM campaigns WHERE channel_id = $1").await?;

    let rows = client.query(&statement, &[&channel_id]).await?;

    let campaigns = rows.iter().map(Campaign::from).collect();

    Ok(campaigns)
}

/// ```text
/// UPDATE campaigns SET budget = $1, validators = $2, title = $3, pricing_bounds = $4, event_submission = $5, ad_units = $6, targeting_rules = $7
/// WHERE id = $8
/// RETURNING id, channel, creator, budget, validators, title, pricing_bounds, event_submission, ad_units, targeting_rules, created, active_from, active_to
/// ```
pub async fn update_campaign(pool: &DbPool, campaign: &Campaign) -> Result<Campaign, PoolError> {
    let client = pool.get().await?;
    let statement = client
        .prepare("UPDATE campaigns SET budget = $1, validators = $2, title = $3, pricing_bounds = $4, event_submission = $5, ad_units = $6, targeting_rules = $7 WHERE id = $8 RETURNING id, channel, creator, budget, validators, title, pricing_bounds, event_submission, ad_units, targeting_rules, created, active_from, active_to")
        .await?;

    let ad_units = Json(&campaign.ad_units);

    let updated_row = client
        .query_one(
            &statement,
            &[
                &campaign.budget,
                &campaign.validators,
                &campaign.title,
                &campaign.pricing_bounds,
                &campaign.event_submission,
                &ad_units,
                &campaign.targeting_rules,
                &campaign.id,
            ],
        )
        .await?;

    Ok(Campaign::from(&updated_row))
}

#[cfg(test)]
mod test {
    use primitives::{
        campaign,
        event_submission::{RateLimit, Rule},
        sentry::campaign_create::ModifyCampaign,
        targeting::Rules,
        util::tests::prep_db::{DUMMY_AD_UNITS, DUMMY_CAMPAIGN},
        EventSubmission, UnifiedNum,
    };
    use std::time::Duration;
    use tokio_postgres::error::SqlState;

    use crate::db::tests_postgres::{setup_test_migrations, DATABASE_POOL};

    use super::*;

    #[tokio::test]
    async fn it_inserts_fetches_and_updates_a_campaign() {
        let database = DATABASE_POOL.get().await.expect("Should get a DB pool");

        setup_test_migrations(database.pool.clone())
            .await
            .expect("Migrations should succeed");

        let campaign = DUMMY_CAMPAIGN.clone();

        let non_existent_campaign = fetch_campaign(database.pool.clone(), &campaign.id)
            .await
            .expect("Should fetch successfully");

        assert_eq!(None, non_existent_campaign);

        let is_inserted = insert_campaign(&database.pool, &campaign)
            .await
            .expect("Should succeed");

        assert!(is_inserted);

        let is_duplicate_inserted = insert_campaign(&database.pool, &campaign).await;

        assert!(matches!(
            is_duplicate_inserted,
            Err(PoolError::Backend(error)) if error.code() == Some(&SqlState::UNIQUE_VIOLATION)
        ));

        let fetched_campaign = fetch_campaign(database.pool.clone(), &campaign.id)
            .await
            .expect("Should fetch successfully");

        assert_eq!(Some(campaign.clone()), fetched_campaign);

        // Update campaign
        {
            let rule = Rule {
                uids: None,
                rate_limit: Some(RateLimit {
                    limit_type: "sid".to_string(),
                    time_frame: Duration::from_millis(20_000),
                }),
            };
            let new_budget = campaign.budget + UnifiedNum::from_u64(1_000_000_000);
            let modified_campaign = ModifyCampaign {
                budget: Some(new_budget),
                validators: None,
                title: Some("Modified Campaign".to_string()),
                pricing_bounds: Some(campaign::PricingBounds {
                    impression: Some(campaign::Pricing {
                        min: 1.into(),
                        max: 10.into(),
                    }),
                    click: Some(campaign::Pricing {
                        min: 0.into(),
                        max: 0.into(),
                    }),
                }),
                event_submission: Some(EventSubmission { allow: vec![rule] }),
                ad_units: Some(DUMMY_AD_UNITS.to_vec()),
                targeting_rules: Some(Rules::new()),
            };

            let applied_campaign = modified_campaign.apply(campaign.clone());

            let updated_campaign = update_campaign(&database.pool, &applied_campaign)
                .await
                .expect("should update");

            assert_eq!(
                applied_campaign, updated_campaign,
                "Postgres should update all modified fields"
            );
        }
    }
}
