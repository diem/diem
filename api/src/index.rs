// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    accounts,
    context::Context,
    events, log,
    metrics::{metrics, status_metrics},
    transactions,
};
use diem_api_types::{Error, Response};

use std::convert::Infallible;
use warp::{
    body::BodyDeserializeError,
    cors::CorsForbidden,
    http::StatusCode,
    reject::{LengthRequired, MethodNotAllowed, PayloadTooLarge, UnsupportedMediaType},
    reply, Filter, Rejection, Reply,
};

const OPEN_API_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/spec.html"));
const OPEN_API_SPEC: &str = include_str!("../doc/openapi.yaml");

pub fn routes(context: Context) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    index(context.clone())
        .or(openapi_spec())
        .or(accounts::routes(context.clone()))
        .or(transactions::routes(context.clone()))
        .or(events::routes(context.clone()))
        .or(context.health_check_route().with(metrics("health_check")))
        // jsonrpc routes must before `recover` and after `index`
        // so that POST '/' can be handled by jsonrpc routes instead of `index` route
        .or(context.jsonrpc_routes().with(metrics("json_rpc")))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["POST", "GET"]),
        )
        .recover(handle_rejection)
        .with(log::logger())
        .with(status_metrics())
}

// GET /openapi.yaml
// GET /spec.html
pub fn openapi_spec() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let spec = warp::path!("openapi.yaml")
        .and(warp::get())
        .map(|| OPEN_API_SPEC)
        .with(metrics("openapi_yaml"));
    let renderer = warp::path!("redoc.standalone.js").and(warp::get()).map(|| {
        warp::http::Response::builder()
            .header("Content-Type", "text/javascript")
            .body(include_str!("../doc/redoc.standalone.js"))
    });
    let html = warp::path!("spec.html")
        .and(warp::get())
        .map(|| reply::html(OPEN_API_HTML))
        .with(metrics("spec_html"));
    spec.or(renderer).or(html)
}

// GET /
pub fn index(context: Context) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(context.filter())
        .and_then(handle_index)
        .with(metrics("get_ledger_info"))
}

pub async fn handle_index(context: Context) -> Result<impl Reply, Rejection> {
    let info = context.get_latest_ledger_info()?;
    Ok(Response::new(info.clone(), &info)?)
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let body;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        body = reply::json(&Error::new(code, "Not Found".to_owned()));
    } else if let Some(error) = err.find::<Error>() {
        code = error.status_code();
        body = reply::json(error);
    } else if let Some(cause) = err.find::<CorsForbidden>() {
        code = StatusCode::FORBIDDEN;
        body = reply::json(&Error::new(code, cause.to_string()));
    } else if let Some(cause) = err.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        body = reply::json(&Error::new(code, cause.to_string()));
    } else if let Some(cause) = err.find::<LengthRequired>() {
        code = StatusCode::LENGTH_REQUIRED;
        body = reply::json(&Error::new(code, cause.to_string()));
    } else if let Some(cause) = err.find::<PayloadTooLarge>() {
        code = StatusCode::PAYLOAD_TOO_LARGE;
        body = reply::json(&Error::new(code, cause.to_string()));
    } else if let Some(cause) = err.find::<UnsupportedMediaType>() {
        code = StatusCode::UNSUPPORTED_MEDIA_TYPE;
        body = reply::json(&Error::new(code, cause.to_string()));
    } else if let Some(cause) = err.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        body = reply::json(&Error::new(code, cause.to_string()));
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        body = reply::json(&Error::new(code, format!("unexpected error: {:?}", err)));
    }
    Ok(reply::with_status(body, code))
}
