use {
    crate::api::{ApiReply, IntoWarpReply},
    anyhow::Result,
    ethcontract::H160,
    serde::Serialize,
    shared::price_estimation::native::NativePriceEstimating,
    std::{convert::Infallible, sync::Arc},
    warp::{Filter, Rejection, hyper::StatusCode, reply::with_status},
};

#[derive(Serialize)]
struct PriceResponse {
    price: f64,
}

fn get_native_prices_request() -> impl Filter<Extract = (H160,), Error = Rejection> + Clone {
    warp::path!("v1" / "token" / H160 / "native_price").and(warp::get())
}

pub fn get_native_price(
    estimator: Arc<dyn NativePriceEstimating>,
) -> impl Filter<Extract = (ApiReply,), Error = Rejection> + Clone {
    get_native_prices_request().and_then(move |token: H160| {
        let estimator = estimator.clone();
        async move {
            let result = estimator.estimate_native_price(token).await;
            let reply = match result {
                Ok(price) => {
                    with_status(warp::reply::json(&PriceResponse { price }), StatusCode::OK)
                }
                Err(err) => err.into_warp_reply(),
            };
            Result::<_, Infallible>::Ok(reply)
        }
    })
}

#[cfg(test)]
mod tests {
    use {super::*, futures::FutureExt, hex_literal::hex, warp::test::request};

    #[test]
    fn native_prices_query() {
        let path = "/v1/token/0xdac17f958d2ee523a2206206994597c13d831ec7/native_price";
        let request = request().path(path).method("GET");
        let result = request
            .filter(&get_native_prices_request())
            .now_or_never()
            .unwrap()
            .unwrap();
        assert_eq!(
            result,
            H160(hex!("dac17f958d2ee523a2206206994597c13d831ec7"))
        );
    }
}
