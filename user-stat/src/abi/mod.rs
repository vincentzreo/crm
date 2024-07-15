use chrono::{TimeZone, Utc};
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{
    pb::{QueryRequest, RawQueryRequest, User},
    ResponseStream, ServiceResult, UserStatsService,
};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        let mut sql = "SELECT email, name FROM user_stats where ".to_string();
        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");
        sql.push_str(&time_conditions);
        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");
        sql.push_str(" AND ");
        sql.push_str(&id_conditions);
        println!("Query: {}", sql);
        self.raw_query(RawQueryRequest { query: sql }).await
    }
    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.inner.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query {}",
                req.query
            )));
        };
        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        "TRUE".to_string()
    } else {
        format!("array{:?} <@ {}", ids, name)
    }
}

fn timestamp_query(name: &str, lower: Option<Timestamp>, upper: Option<Timestamp>) -> String {
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }
    if lower.is_none() {
        let after = ts_to_utc(upper.unwrap());
        return format!("{} <= '{}'", name, after.to_rfc3339());
    }
    if upper.is_none() {
        let before = ts_to_utc(lower.unwrap());
        return format!("{} >= '{}'", name, before.to_rfc3339());
    }
    format!(
        "{} BETWEEN '{}' AND '{}'",
        name,
        ts_to_utc(upper.unwrap()).to_rfc3339(),
        ts_to_utc(lower.unwrap()).to_rfc3339()
    )
}

fn ts_to_utc(ts: Timestamp) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

#[cfg(test)]
mod tests {
    use futures::StreamExt as _;

    use crate::{
        pb::QueryRequestBuilder,
        test_utils::{id, tq},
    };

    use super::*;
    #[tokio::test]
    async fn raw_query_should_work() -> anyhow::Result<()> {
        let (_tdb, svc) = UserStatsService::new_for_test().await?;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query: "SELECT * from user_stats where created_at > '2024-07-01' LIMIT 5;"
                    .to_string(),
            })
            .await?
            .into_inner();
        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }
        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> anyhow::Result<()> {
        let (_tdb, svc) = UserStatsService::new_for_test().await?;
        let query = QueryRequestBuilder::default()
            .timestamp(("created_at".to_string(), tq(None, Some(30))))
            .timestamp(("last_visited_at".to_string(), tq(Some(15), Some(30))))
            .id(("viewed_but_not_started".to_string(), id(&[272045])))
            .build()?;
        let mut stream = svc.query(query).await?.into_inner();
        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }
        Ok(())
    }
}
