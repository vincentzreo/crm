use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};
use tracing::info;

use crate::{
    pb::{QueryRequest, QueryRequestBuilder, RawQueryRequest, TimeQuery, User},
    ResponseStream, ServiceResult, UserStatsService,
};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        let sql = query.to_string();
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

impl fmt::Display for QueryRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sql = "SELECT email, name FROM user_stats where ".to_string();
        let time_conditions = self
            .timestamps
            .iter()
            .map(|(k, v)| timestamp_query(k, v.lower.as_ref(), v.upper.as_ref()))
            .join(" AND ");
        sql.push_str(&time_conditions);
        let id_conditions = self
            .ids
            .iter()
            .map(|(k, v)| ids_query(k, &v.ids))
            .join(" AND ");
        if !id_conditions.is_empty() {
            if !self.timestamps.is_empty() {
                sql.push_str(" AND ");
            }
            sql.push_str(&id_conditions);
        }
        info!("Query: {}", sql);
        write!(f, "{}", sql)
    }
}

impl QueryRequest {
    pub fn new_with_dt(name: &str, lower: DateTime<Utc>, upper: DateTime<Utc>) -> Self {
        let ts = Timestamp {
            seconds: lower.timestamp(),
            nanos: 0,
        };
        let ts1 = Timestamp {
            seconds: upper.timestamp(),
            nanos: 0,
        };
        let tq = TimeQuery {
            lower: Some(ts),
            upper: Some(ts1),
        };
        QueryRequestBuilder::default()
            .timestamp((name.to_string(), tq))
            .build()
            .expect("Failed to build query request")
    }
}

fn ids_query(name: &str, ids: &[u32]) -> String {
    if ids.is_empty() {
        "TRUE".to_string()
    } else {
        format!("array{:?} <@ {}", ids, name)
    }
}

fn timestamp_query(name: &str, lower: Option<&Timestamp>, upper: Option<&Timestamp>) -> String {
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
        ts_to_utc(lower.unwrap()).to_rfc3339(),
        ts_to_utc(upper.unwrap()).to_rfc3339()
    )
}

fn ts_to_utc(ts: &Timestamp) -> chrono::DateTime<Utc> {
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

    #[test]
    fn query_to_string_should_work() {
        let query = QueryRequestBuilder::default()
            /* .timestamp(("created_at".to_string(), tq(None, Some(30))))
            .timestamp(("last_visited_at".to_string(), tq(Some(15), Some(30)))) */
            .id(("viewed_but_not_started".to_string(), id(&[272045])))
            .build()
            .unwrap();
        let sql = query.to_string();
        println!("{}", sql);
    }

    #[test]
    fn query_request_to_string_should_work() {
        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let sql = query.to_string();
        assert_eq!(sql, "SELECT email, name FROM user_stats where created_at BETWEEN '2024-01-01T00:00:00+00:00' AND '2024-01-02T00:00:00+00:00'");
    }

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
