extern crate actix_web;
extern crate actix;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use actix_web::{http, server, App, HttpRequest, HttpResponse, Json, State};
use std::sync::Arc;
use std::sync::Mutex;
//use futures::{Future, Stream};

struct AppState {
    state: Arc<Mutex<StateTotal>>,
}

struct StateTotal {
    total: i64,
    transactions: Vec<String>
}

#[derive(Serialize, Deserialize)]
struct SpentRequest {
    amount: f64,
}

#[derive(Serialize)]
struct SpentResponse {
    total: String
}

#[derive(Serialize)]
struct SpentTotalResponse {
    total: String,
    transactions: Vec<String>
}

fn main() {
    let state = Arc::new(Mutex::new(StateTotal{total: 0, transactions: Vec::new()}));

    let sys = actix::System::new("api");

    server::new(move || {
        App::with_state(AppState{ state: state.clone()})
            .resource("/spent", |r| {
                r.method(http::Method::POST).with(spent)
        }) //TODO: correctly access state with get request
            .resource("/spent-total", |r| r.method(http::Method::GET).with(spent_total))
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
            //format i64 into $xx.xx

            return HttpResponse::Ok().content_type("application/json").body(format_money(i.total.clone().to_string()));
        },
        Err(_) => return HttpResponse::InternalServerError().into(),
    }
}


//get
fn spent_total(state: State<AppState>, _req: &HttpRequest<AppState>) -> HttpResponse {
    match state.state.lock(){
        Ok(mut i) => {
            return HttpResponse::Ok().content_type("application/json").body(i.total.clone().to_string());
        },
        Err(_) => return HttpResponse::InternalServerError().into(),
    };
}

fn format_money(mut input: String) -> String {
    if input.len() < 1 {
        input
    } else if input.len() < 2 {
        "$0.0".to_string() + input.as_str()
    } else if input.len() < 3 {
        "$0.".to_string() + input.as_str()
    } else {
        let mut output = "$".to_string() +input.as_str();
        //TODO: add decimal in correct position
    }
}