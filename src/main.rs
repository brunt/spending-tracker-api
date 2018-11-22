extern crate actix;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use actix_web::{http, server, App, HttpRequest, HttpResponse, Json, State};
use std::sync::Arc;
use std::sync::Mutex;

struct AppState {
    state: Arc<Mutex<StateTotal>>,
}

struct StateTotal {
    total: i64,
    transactions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SpentRequest {
    amount: f64,
}

#[derive(Serialize)]
struct SpentResponse {
    total: String,
}

#[derive(Serialize)]
struct SpentTotalResponse {
    total: String,
    transactions: Vec<String>,
}

fn main() {
    let state = Arc::new(Mutex::new(StateTotal {
        total: 0,
        transactions: Vec::new(),
    }));

    let sys = actix::System::new("api");

    server::new(move || {
        App::with_state(AppState {
            state: state.clone(),
        }).resource("/spent", |r| r.method(http::Method::POST).with(spent))
        .resource("/spent-total", |r| r.f(spent_total))
        .resource("/reset", |r| r.f(reset))
    }).bind("0.0.0.0:8001")
    .expect("Address already in use")
    .shutdown_timeout(5)
    .start();
    println!("app started on port 8001");
    let _ = sys.run();
}

//send a value which must be parsed and added to the total
//we take in an f64, we store an i64, and we return a String
fn spent(state: State<AppState>, req: Json<SpentRequest>) -> HttpResponse {
    let spent = req.into_inner();
    let add = (spent.amount * 100.0).round() as i64;
    match state.state.lock() {
        Ok(mut i) => {
            i.total += add;
            i.transactions.push(spent.amount.to_string());
            match serde_json::to_string(&SpentResponse {
                total: format_money(i.total.to_string()),
            }) {
                Ok(s) => return HttpResponse::Ok().content_type("application/json").body(s),
                Err(_) => HttpResponse::InternalServerError().into(),
            }
        }
        Err(_) => return HttpResponse::InternalServerError().into(),
    }
}

fn spent_total(req: &HttpRequest<AppState>) -> HttpResponse {
    match req.state().state.lock() {
        Ok(i) => match serde_json::to_string(&SpentTotalResponse {
            total: format_money(i.total.to_string()),
            transactions: i.transactions.clone(),
        }) {
            Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
            Err(_) => HttpResponse::InternalServerError().into(),
        },
        Err(_) => return HttpResponse::InternalServerError().into(),
    }
}

fn reset(req: &HttpRequest<AppState>) -> HttpResponse {
    match req.state().state.lock() {
        Ok(mut i) => {
            i.total = 0;
            i.transactions = Vec::new();
            match serde_json::to_string(&SpentTotalResponse {
                total: format_money(i.total.to_string()),
                transactions: i.transactions.clone(),
            }) {
                Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
                Err(_) => HttpResponse::InternalServerError().into(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

//the chatbot client is doing input validation so I'm not bothered by not having any here
fn format_money(input: String) -> String {
    match input.len() {
        x if x < 1 => input,
        x if x < 2 => "$0.0".to_string() + input.as_str(),
        x if x < 3 => "$0.".to_string() + input.as_str(),
        _ => {
            let mut output = "$".to_string() + input.as_str();
            output.insert(input.len() - 1, '.');
            output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_format_money() {
        assert_eq!(format_money("".to_string()), "".to_string());
        assert_eq!(format_money("1".to_string()), "$0.01".to_string());
        assert_eq!(format_money("11".to_string()), "$0.11".to_string());
        assert_eq!(format_money("111".to_string()), "$1.11".to_string());
        assert_eq!(format_money("1111".to_string()), "$11.11".to_string());
    }
}
