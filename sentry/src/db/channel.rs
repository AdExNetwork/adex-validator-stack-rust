use crate::db::DbPool;
use bb8::RunError;
use bb8_postgres::tokio_postgres::types::{accepts, FromSql, ToSql, Type};
use chrono::{DateTime, Utc};
use primitives::{Channel, ChannelId, ValidatorId};
use std::error::Error;
use std::str::FromStr;

struct TotalPages(pub u64);
impl<'a> FromSql<'a> for TotalPages {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let str_slice = <&str as FromSql>::from_sql(ty, raw)?;

        Ok(Self(u64::from_str(str_slice)?))
    }

    // Use a varchar or text, since otherwise `int8` fails deserialization
    accepts!(VARCHAR, TEXT);
}

pub async fn get_channel_by_id(
    pool: &DbPool,
    id: &ChannelId,
) -> Result<Option<Channel>, RunError<bb8_postgres::tokio_postgres::Error>> {
    pool
        .run(move |connection| {
            async move {
                match connection.prepare("SELECT id, creator, deposit_asset, deposit_amount, valid_until, spec FROM channels WHERE id = $1 LIMIT 1").await {
                    Ok(select) => match connection.query(&select, &[&id]).await {
                        Ok(results) => Ok((results.get(0).map(Channel::from), connection)),
                        Err(e) => Err((e, connection)),
                    },
                    Err(e) => Err((e, connection)),
                }
            }
        })
        .await
}

pub async fn get_channel_by_id_and_validator(
    pool: &DbPool,
    id: &ChannelId,
    validator_id: &ValidatorId,
) -> Result<Option<Channel>, RunError<bb8_postgres::tokio_postgres::Error>> {
    pool
        .run(move |connection| {
            async move {
                let validator = serde_json::Value::from_str(&format!(r#"[{{"id": "{}"}}]"#, validator_id)).expect("Not a valid json");
                let query = "SELECT id, creator, deposit_asset, deposit_amount, valid_until, spec FROM channels WHERE id = $1 AND spec->'validators' @> $2 LIMIT 1";
                match connection.prepare(query).await {
                    Ok(select) => {
                        match connection.query(&select, &[&id, &validator]).await {
                            Ok(results) => Ok((results.get(0).map(Channel::from), connection)),
                            Err(e) => Err((e, connection)),
                        }
                    },
                    Err(e) => Err((e, connection)),
                }
            }
        })
        .await
}

pub async fn insert_channel(
    pool: &DbPool,
    channel: &Channel,
) -> Result<bool, RunError<bb8_postgres::tokio_postgres::Error>> {
    pool
        .run(move |connection| {
            async move {
                match connection.prepare("INSERT INTO channels (id, creator, deposit_asset, deposit_amount, valid_until, spec) values ($1, $2, $3, $4, $5, $6)").await {
                    Ok(stmt) => match connection.execute(&stmt, &[&channel.id, &channel.creator, &channel.deposit_asset, &channel.deposit_amount, &channel.valid_until, &channel.spec]).await {
                        Ok(row) => {
                            let inserted = row == 1;
                            Ok((inserted, connection))
                        },
                        Err(e) => Err((e, connection)),
                    },
                    Err(e) => Err((e, connection)),
                }
            }
        })
        .await
}

pub async fn list_channels(
    pool: &DbPool,
    skip: u64,
    limit: u32,
    creator: &Option<String>,
    validator: &Option<ValidatorId>,
    valid_until_ge: &DateTime<Utc>,
) -> Result<Vec<Channel>, RunError<bb8_postgres::tokio_postgres::Error>> {
    let validator = validator.as_ref().map(|validator_id| {
        serde_json::Value::from_str(&format!(r#"[{{"id": "{}"}}]"#, validator_id))
            .expect("Not a valid json")
    });
    let (where_clauses, params) =
        channel_list_query_params(creator, validator.as_ref(), valid_until_ge);

    pool
        .run(move |connection| {
            async move {
                // To understand why we use Order by, see Postgres Documentation: https://www.postgresql.org/docs/8.1/queries-limit.html
                let statement = format!("SELECT id, creator, deposit_asset, deposit_amount, valid_until, spec FROM channels WHERE {} ORDER BY id DESC LIMIT {} OFFSET {}", where_clauses.join(" AND "), limit, skip);
                match connection.prepare(&statement).await {
                    Ok(stmt) => {
                        match connection.query(&stmt, params.as_slice()).await {
                            Ok(rows) => {
                                let channels = rows.iter().map(Channel::from).collect();

                                Ok((channels, connection))
                            },
                            Err(e) => Err((e, connection)),
                        }
                    },
                    Err(e) => Err((e, connection)),
                }
            }
        })
        .await
}

pub async fn list_channels_total_pages(
    pool: &DbPool,
    creator: &Option<String>,
    validator: &Option<ValidatorId>,
    valid_until_ge: &DateTime<Utc>,
) -> Result<u64, RunError<bb8_postgres::tokio_postgres::Error>> {
    let validator = validator.as_ref().map(|validator_id| {
        serde_json::Value::from_str(&format!(r#"[{{"id": "{}"}}]"#, validator_id))
            .expect("Not a valid json")
    });
    let (where_clauses, params) =
        channel_list_query_params(creator, validator.as_ref(), valid_until_ge);

    pool.run(move |connection| {
        async move {
            let statement = format!(
                "SELECT COUNT(id)::varchar FROM channels WHERE {}",
                where_clauses.join(" AND ")
            );
            match connection.prepare(&statement).await {
                Ok(stmt) => match connection.query_one(&stmt, params.as_slice()).await {
                    Ok(row) => Ok((row.get::<_, TotalPages>(0).0, connection)),
                    Err(e) => Err((e, connection)),
                },
                Err(e) => Err((e, connection)),
            }
        }
    })
    .await
}

fn channel_list_query_params<'a>(
    creator: &'a Option<String>,
    validator: Option<&'a serde_json::Value>,
    valid_until_ge: &'a DateTime<Utc>,
) -> (Vec<String>, Vec<&'a (dyn ToSql + Sync)>) {
    let mut where_clauses = vec!["valid_until >= $1".to_string()];
    let mut params: Vec<&(dyn ToSql + Sync)> = vec![valid_until_ge];

    if let Some(creator) = creator {
        where_clauses.push(format!("creator = ${}", params.len() + 1));
        params.push(creator);
    }

    if let Some(validator) = validator {
        where_clauses.push(format!("spec->'validators' @> ${}", params.len() + 1));
        params.push(validator);
    }

    (where_clauses, params)
}