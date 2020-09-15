#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rust_embed;

use actix_cors::Cors;
use actix_web::{body::Body, get, post, web, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetrics;
use mime_guess::from_path;
use rusty_money::Iso::*;
use rusty_money::{Currency, Money};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};

mod category;
use category::Category;

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;

struct AppState {
    state: Arc<RwLock<StateTotal>>,
}

struct StateTotal {
    budget: Money,
    total: Money,
    transactions: Vec<(String, Category)>,
}

#[derive(Serialize, Deserialize)]
struct SpentRequest {
    amount: f64,
    category: Option<Category>,
}

#[derive(Serialize)]
struct SpentResponse {
    total: String,
}

#[derive(Serialize)]
struct SpentTotalResponse {
    budget: String,
    total: String,
    transactions: Vec<(String, Category)>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Arc::new(RwLock::new(StateTotal {
        budget: Money::from_minor(50000, Currency::get(USD)),
        total: Money::from_minor(0, Currency::get(USD)),
        transactions: Vec::new(),
    }));

    let prometheus = PrometheusMetrics::new("spending", Some("/metrics"), None);
    HttpServer::new(move || {
        App::new()
            .data(AppState {
                state: state.to_owned(),
            })
            .wrap(
                Cors::new()
                    .allowed_origin("localhost")
                    .allowed_methods(vec!["GET", "POST"])
                    .finish(),
            )
            .wrap(prometheus.clone())
            .service(
                web::resource("/spent")
                    .route(web::post().to(spent))
                    .route(web::get().to(spent_total)),
            )
            .service(reset)
            .service(set_budget)
            .service(index)
            .service(dist)
    })
    .bind("0.0.0.0:8001")?
    .run()
    .await
}

async fn spent(state: web::Data<AppState>, req: web::Json<SpentRequest>) -> HttpResponse {
    let spent = req.into_inner();
    let add = Money::from_minor((spent.amount * 100.0).round() as i64, Currency::get(USD));
    match state.state.write() {
        Ok(mut state) => {
            state.total += add.clone();
            state
                .transactions
                .push((add.to_string(), spent.category.unwrap_or(Category::Other)));
            match serde_json::to_string(&SpentResponse {
                total: (state.budget.clone() - state.total.clone()).to_string(),
            }) {
                Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
                Err(_) => HttpResponse::InternalServerError().into(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

async fn spent_total(req: web::Data<AppState>) -> HttpResponse {
    match req.state.read() {
        Ok(i) => match serde_json::to_string(&SpentTotalResponse {
            budget: i.budget.clone().to_string(),
            total: i.total.clone().to_string(),
            transactions: i.transactions.clone(),
        }) {
            Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
            Err(_) => HttpResponse::InternalServerError().into(),
        },
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[get("/reset")]
async fn reset(req: web::Data<AppState>) -> HttpResponse {
    match req.state.write() {
        Ok(mut i) => {
            i.budget = Money::from_minor(50000, Currency::get(USD));
            i.total = Money::from_minor(0, Currency::get(USD));
            i.transactions = Vec::new();
            match serde_json::to_string(&SpentTotalResponse {
                budget: i.budget.clone().to_string(),
                total: i.total.clone().to_string(),
                transactions: Vec::new(),
            }) {
                Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
                Err(_) => HttpResponse::InternalServerError().into(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[post("/budget")]
async fn set_budget(state: web::Data<AppState>, req: web::Json<SpentRequest>) -> HttpResponse {
    match state.state.write() {
        Ok(mut i) => {
            i.budget = Money::from_minor((req.amount * 100.0).round() as i64, Currency::get(USD));
            match serde_json::to_string(&SpentTotalResponse {
                budget: i.budget.clone().to_string(),
                total: i.total.clone().to_string(),
                transactions: i.transactions.clone(),
            }) {
                Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
                Err(_) => HttpResponse::InternalServerError().into(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };
            HttpResponse::Ok()
                .content_type(from_path(path).first_or_octet_stream().to_string())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[get("/")]
async fn index(_req: web::Data<AppState>) -> HttpResponse {
    handle_embedded_file("index.html")
}

#[get("/dist/{_:.*}")]
async fn dist(path: web::Path<String>) -> HttpResponse {
    handle_embedded_file(&path.0)
}
