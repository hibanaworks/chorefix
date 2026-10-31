//! Loopback fixtures for the original comparison clients. No model requests.
use crate::{
    evidence::{Result, save},
    model::{EFFORT, MODEL},
};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use serde_json::{Value, json};
use std::{
    cell::Cell,
    convert::Infallible,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{io::AsyncWriteExt, net::TcpListener, task::LocalSet};

type Reply = Response<Full<Bytes>>;
// Host framing budget, separate from protocol authority and model context limits.
const MAX_REQUEST_BYTES: usize = 64 * 1024 * 1024;

fn reply(status: StatusCode, value: &Value) -> Reply {
    let mut response = Response::new(Full::new(Bytes::from(value.to_string())));
    *response.status_mut() = status;
    response.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    response
}

fn failure(status: StatusCode, error: impl std::fmt::Display) -> Reply {
    reply(
        status,
        &json!({"error":{"type":"model_transport_error","message":error.to_string()}}),
    )
}

async fn handle<F, Fut>(
    request: Request<Incoming>,
    directory: PathBuf,
    forward: F,
) -> std::result::Result<Reply, Infallible>
where
    F: FnOnce(Value, PathBuf) -> Fut,
    Fut: Future<Output = Result<(StatusCode, Value)>>,
{
    let started = Instant::now();
    let started_unix_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock precedes Unix epoch")
        .as_nanos()
        .to_string();
    if request.method() != Method::POST || request.uri().path() != "/v1/chat/completions" {
        return Ok(failure(
            StatusCode::NOT_FOUND,
            "expected POST /v1/chat/completions",
        ));
    }
    let body = match Limited::new(request.into_body(), MAX_REQUEST_BYTES)
        .collect()
        .await
    {
        Ok(body) => body.to_bytes(),
        Err(error) => return Ok(failure(StatusCode::PAYLOAD_TOO_LARGE, error)),
    };
    let chat: Value = match serde_json::from_slice(&body) {
        Ok(chat) => chat,
        Err(error) => return Ok(failure(StatusCode::BAD_REQUEST, error)),
    };
    let outcome = async {
        fs::create_dir(&directory)?;
        save(&directory.join("client-request.json"), &chat)?;
        let prepared = match super::request(&chat) {
            Ok(prepared) => prepared,
            Err(error) => return Ok(failure(StatusCode::BAD_REQUEST, error)),
        };
        save(&directory.join("responses-request.json"), &prepared)?;
        save(
            &directory.join("app-server-history.json"),
            &json!({"items":super::history::items(&prepared)?}),
        )?;
        let (status, response) = forward(prepared, directory.clone()).await?;
        save(&directory.join("responses-response.json"), &response)?;
        save(
            &directory.join("transport.json"),
            &json!({"http_status":status.as_u16(),
            "requested_model":MODEL,"requested_effort":EFFORT,
            "returned_model":response["model"],"returned_effort":response["reasoning"]["effort"],
            "usage":response["usage"],"started_unix_ns":started_unix_ns,
            "completed_unix_ns":SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos().to_string(),
            "elapsed_seconds":started.elapsed().as_secs_f64(),"scope":"model_api_transport"}),
        )?;
        if !status.is_success() {
            return Ok(reply(status, &response));
        }
        let completion = super::response(&response)?;
        save(&directory.join("client-response.json"), &completion)?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(reply(StatusCode::OK, &completion))
    }
    .await;
    Ok(match outcome {
        Ok(response) => response,
        Err(error) => {
            let _ = save(
                &directory.join("transport-error.json"),
                &json!({"error":error.to_string(),"scope":"model_api_transport"}),
            );
            failure(StatusCode::BAD_GATEWAY, error)
        }
    })
}

async fn connections<F, Fut>(listener: TcpListener, output: PathBuf, forward: F) -> Result<()>
where
    F: Fn(Value, PathBuf) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<(StatusCode, Value)>> + 'static,
{
    let next_request = Rc::new(Cell::new(0u64));
    LocalSet::new()
        .run_until(async move {
            loop {
                let (stream, _) = listener.accept().await?;
                let next_request = next_request.clone();
                let output = output.clone();
                let forward = forward.clone();
                tokio::task::spawn_local(async move {
                    let service = service_fn(move |request| {
                        let next = next_request
                            .get()
                            .checked_add(1)
                            .expect("request number exhausted");
                        next_request.set(next);
                        handle(
                            request,
                            output.join(format!("request-{next:08}")),
                            forward.clone(),
                        )
                    });
                    if let Err(error) = http1::Builder::new()
                        .serve_connection(TokioIo::new(stream), service)
                        .await
                    {
                        eprintln!("comparison client connection: {error}");
                    }
                });
            }
        })
        .await
}

#[cfg(test)]
mod tests;
