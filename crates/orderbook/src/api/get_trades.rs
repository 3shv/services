use {
    crate::{
        api::{ApiReply, error},
        database::{
            Postgres,
            trades::{TradeFilter, TradeRetrieving},
        },
    },
    anyhow::{Context, Result},
    model::order::OrderUid,
    primitive_types::H160,
    serde::Deserialize,
    std::convert::Infallible,
    warp::{Filter, Rejection, hyper::StatusCode, reply::with_status},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Query {
    pub order_uid: Option<OrderUid>,
    pub owner: Option<H160>,
}

#[derive(Debug, Eq, PartialEq)]
enum TradeFilterError {
    InvalidFilter(String),
}

impl Query {
    fn trade_filter(&self) -> TradeFilter {
        TradeFilter {
            order_uid: self.order_uid,
            owner: self.owner,
        }
    }

    fn validate(&self) -> Result<TradeFilter, TradeFilterError> {
        match (self.order_uid.as_ref(), self.owner.as_ref()) {
            (Some(_), None) | (None, Some(_)) => Ok(self.trade_filter()),
            _ => Err(TradeFilterError::InvalidFilter(
                "Must specify exactly one of owner or orderUid.".to_owned(),
            )),
        }
    }
}

fn get_trades_request()
-> impl Filter<Extract = (Result<TradeFilter, TradeFilterError>,), Error = Rejection> + Clone {
    warp::path!("v1" / "trades")
        .and(warp::get())
        .and(warp::query::<Query>())
        .map(|query: Query| query.validate())
}

pub fn get_trades(db: Postgres) -> impl Filter<Extract = (ApiReply,), Error = Rejection> + Clone {
    get_trades_request().and_then(move |request_result| {
        let database = db.clone();
        async move {
            Result::<_, Infallible>::Ok(match request_result {
                Ok(trade_filter) => {
                    let result = database.trades(&trade_filter).await.context("get_trades");
                    match result {
                        Ok(reply) => with_status(warp::reply::json(&reply), StatusCode::OK),
                        Err(err) => {
                            tracing::error!(?err, "get_trades");
                            crate::api::internal_error_reply()
                        }
                    }
                }
                Err(TradeFilterError::InvalidFilter(msg)) => {
                    let err = error("InvalidTradeFilter", msg);
                    with_status(err, StatusCode::BAD_REQUEST)
                }
            })
        }
    })
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        hex_literal::hex,
        primitive_types::H160,
        warp::test::{RequestBuilder, request},
    };

    #[tokio::test]
    async fn get_trades_request_ok() {
        let trade_filter = |request: RequestBuilder| async move {
            let filter = get_trades_request();
            request.method("GET").filter(&filter).await
        };

        let owner = H160::from_slice(&hex!("0000000000000000000000000000000000000001"));
        let owner_path = format!("/v1/trades?owner=0x{owner:x}");
        let result = trade_filter(request().path(owner_path.as_str()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.owner, Some(owner));
        assert_eq!(result.order_uid, None);

        let uid = OrderUid([1u8; 56]);
        let order_uid_path = format!("/v1/trades?orderUid={uid}");
        let result = trade_filter(request().path(order_uid_path.as_str()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.owner, None);
        assert_eq!(result.order_uid, Some(uid));
    }

    #[tokio::test]
    async fn get_trades_request_err() {
        let trade_filter = |request: RequestBuilder| async move {
            let filter = get_trades_request();
            request.method("GET").filter(&filter).await
        };

        let owner = H160::from_slice(&hex!("0000000000000000000000000000000000000001"));
        let uid = OrderUid([1u8; 56]);
        let path = format!("/v1/trades?owner=0x{owner:x}&orderUid={uid}");

        let result = trade_filter(request().path(path.as_str())).await.unwrap();
        assert!(result.is_err());

        let path = "/v1/trades";
        let result = trade_filter(request().path(path)).await.unwrap();
        assert!(result.is_err());
    }
}
