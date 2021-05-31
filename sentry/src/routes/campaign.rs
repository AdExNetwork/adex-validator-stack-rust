use crate::{
    success_response, Application, Auth, ResponseError, RouteParams, Session,
    db::{
        spendable::fetch_spendable,
        event_aggregate::latest_new_state_v5,
        DbPool
    },
};
use hyper::{Body, Request, Response};
use primitives::{
    adapter::Adapter,
    sentry::{
        campaign_create::CreateCampaign,
        SuccessResponse
    },
    Campaign, CampaignId, UnifiedNum, ChannelId
};
use redis::aio::MultiplexedConnection;
use deadpool_postgres::PoolError;
use slog::error;
use tokio_postgres::error::SqlState;
use crate::db::campaign::{campaign_exists, update_campaign, insert_campaign, get_campaigns_for_channel};

pub async fn create_campaign<A: Adapter>(
    req: Request<Body>,
    app: &Application<A>,
) -> Result<Response<Body>, ResponseError> {
    let body = hyper::body::to_bytes(req.into_body()).await?;

    let campaign = serde_json::from_slice::<CreateCampaign>(&body)
        .map_err(|e| ResponseError::FailedValidation(e.to_string()))?
        // create the actual `Campaign` with random `CampaignId`
        .into_campaign();


    // TODO AIP#61: Validate Campaign

    let error_response = ResponseError::BadRequest("err occurred; please try again later".into());

    // insert Campaign

    match insert_or_modify_campaign(&app.pool, &campaign, &app.redis).await {
        Err(error) => {
            // error!(&app.logger, "{}", &error; "module" => "create_channel");
            return Err(ResponseError::Conflict("channel already exists".to_string()));
        }
        Ok(false) => Err(error_response),
        _ => Ok(()),
    }?;

    let create_response = SuccessResponse { success: true };

    Ok(success_response(serde_json::to_string(&campaign)?))
}

// TODO: Double check redis calls
async fn get_spent_for_campaign(redis: &MultiplexedConnection, id: CampaignId) -> Result<UnifiedNum, PoolError> {
    let key = format!("adexCampaign:campaignSpent:{}", id);
    // campaignSpent tracks the portion of the budget which has already been spent
    let campaign_spent = match redis::cmd("GET")
    .arg(&key)
    .query_async::<_, Option<String>>(&mut redis.clone())
    .await?
    {
        Some(spent) => UnifiedNum::from(spent),
        // TODO: Double check if this is true
        // If the campaign is just being inserted, there would be no entry therefore no funds would be spent
        None => UnifiedNum::from(0)
    };
    Ok(campaign_spent)
}

async fn update_remaining_for_campaign(redis: &MultiplexedConnection, id: CampaignId, amount: UnifiedNum) -> Result<bool, PoolError> {
    // update a key in Redis for the remaining spendable amount
    let key = format!("adexCampaign:remainingSpendable:{}", id);
    redis::cmd("SET")
        .arg(&key)
        .arg(amount.to_u64())
        .query_async(&mut redis.clone())
        .await?;
    Ok(true)
}

async fn update_remaining_for_channel(redis: &MultiplexedConnection, id: ChannelId, amount: UnifiedNum) -> Result<bool, PoolError> {
    let key = format!("adexChannel:remaining:{}", id);
    redis::cmd("SET")
        .arg(&key)
        .arg(amount.to_u64())
        .query_async(&mut redis.clone())
        .await?;
    Ok(true)
}

async fn get_campaigns_remaining_sum(redis: &MultiplexedConnection, pool: &DbPool, campaign: &Campaign) -> Result<UnifiedNum, PoolError> {
    let campaigns_for_channel = get_campaigns_for_channel(&pool, &campaign).await?;
    let sum_of_campaigns_remaining = campaigns_for_channel
        .into_iter()
        .map(async |c| {
            let spent = get_spent_for_campaign(&redis, c.id).await?;
            let remaining = c.budget - spent;
            remaining
        })
        .sum();
    Ok(sum_of_campaigns_remaining)
}

pub async fn insert_or_modify_campaign(pool: &DbPool, campaign: &Campaign, redis: &MultiplexedConnection) -> Result<bool, ResponseError> {
    let campaign_spent = get_spent_for_campaign(&redis, campaign.id).await?;

    // Check if we haven't exceeded the budget yet
    if campaign.budget <= campaign_spent {
        return Err(ResponseError::FailedValidation("No more budget available for spending".into()));
    }

    let remaining_spendable_campaign = campaign.budget - campaign_spent;
    update_remaining_for_campaign(&redis, campaign.id, remaining_spendable_campaign).await?;


    // Getting the latest new state from Postgres
    let latest_new_state = latest_new_state_v5(&pool, &campaign.channel, "").await?;
    // Gets the latest Spendable for this (spender, channelId) pair
    let latest_spendable = fetch_spendable(pool.clone(), &campaign.creator, &campaign.channel.id()).await?;

    let total_deposited = latest_spendable.deposit.total;
    let total_spent = if let Some(lns) = latest_new_state {
        lns.msg.into_inner().spenders[campaign.creator]
    } else {
        0
    };

    let total_remaining = total_deposited - total_spent;

    update_remaining_for_channel(&redis, campaign.channel.id(), total_remaining).await?;

    if campaign_exists(&pool, &campaign).await? {
        let campaigns_remaining_sum = get_campaigns_remaining_sum(&redis, &pool, &campaign).await?;
        if campaigns_remaining_sum > total_remaining {
            return Err(ResponseError::Conflict("Remaining for campaigns exceeds total remaining for channel".into()));
        }
        return update_campaign(&pool, &campaign).await?
    }
    return insert_campaign(&pool, &campaign).await?;

    // *NOTE*: When updating campaigns make sure sum(campaigns.map(getRemaining)) <= totalDepoisted - totalspent
    // !WARNING!: totalSpent != sum(campaign.map(c => c.spending)) therefore we must always calculate remaining funds based on total_deposit - lastApprovedNewState.spenders[user]
    // *NOTE*: To close a campaign set campaignBudget to campaignSpent so that spendable == 0
}
