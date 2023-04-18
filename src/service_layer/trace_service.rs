use crate::requests::requests;
use crate::{data_access_layer, AppState};
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use std::sync::Arc;

pub async fn record_trace<B>(
    State(state): State<Arc<AppState>>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let mut trace = requests::TraceRequest {
        trace_id: None,
        method: request.method().to_string(),
        uri: request.uri().to_string(),
        user_agent: None,
    };

    // todo : clean this
    if let Some(trace_id) = request.headers().get("trace") {
        let trace_id = trace_id
            .to_str()
            .unwrap_or("")
            .parse::<usize>()
            .unwrap_or(0);
        trace.trace_id = Some(trace_id)
    }
    if let Some(user_agent) = request.headers().get("user-agent") {
        let user_agent = user_agent.to_str().unwrap_or("").to_string();
        trace.user_agent = Some(user_agent);
    }

    match data_access_layer::trace_dal::create_trace(&state, trace.clone()) {
        Ok(_) => (),
        Err(e) => println!("failed recording trace {:?} ", trace),
    }
    let response = next.run(request).await;

    response
}
