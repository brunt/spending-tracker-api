extern crate actix_cors;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rust_embed;
extern crate mime_guess;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, body::Body};
use mime_guess::from_path;
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
    total: i64,
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
    total: String,
    transactions: Vec<(String, Category)>,
}

fn main() -> std::io::Result<()> {
    let state = Arc::new(RwLock::new(StateTotal {
        total: 0,
        transactions: Vec::new(),
    }));

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                state: state.to_owned(),
            })
            .wrap(
                Cors::new()
                    .allowed_origin("localhost")
                    .allowed_methods(vec!["GET", "POST"]),
            )
            .service(web::resource("/reset").route(web::get().to(reset)))
            .service(
                web::resource("/spent")
                    .route(web::post().to(spent))
                    .route(web::get().to(spent_total)),
            )
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("{_:.*}").route(web::get().to(dist)))
    })
    .bind("0.0.0.0:8001")?
    .run()
}

//send a value which must be parsed and added to the total
//we take in an f64, we store an i64, and we return a String
fn spent(state: web::Data<AppState>, req: web::Json<SpentRequest>) -> HttpResponse {
    let spent = req.into_inner();
    let add = (spent.amount * 100.0).round() as i64;
    match state.state.write() {
        Ok(mut i) => {
            i.total += add;
            i.transactions.push((
                spent.amount.to_string(),
                spent.category.unwrap_or(Category::Other),
            ));
            match serde_json::to_string(&SpentResponse {
                total: format_money(i.total.to_string()),
            }) {
                Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
                Err(_) => HttpResponse::InternalServerError().into(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

fn spent_total(req: web::Data<AppState>) -> HttpResponse {
    match req.state.read() {
        Ok(i) => match serde_json::to_string(&SpentTotalResponse {
            total: format_money(i.total.to_string()),
            transactions: i.transactions.clone(),
        }) {
            Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
            Err(_) => HttpResponse::InternalServerError().into(),
        },
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

fn reset(req: web::Data<AppState>) -> HttpResponse {
    match req.state.write() {
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
        x if x < 2 => format!("$0.0{}", input),
        x if x < 3 => format!("$0.{}", input),
        _ => {
            let mut output = format!("${}", input);
            output.insert(input.len() - 1, '.');
            output
        }
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

fn index(_req: web::Data<AppState>) -> HttpResponse {
    handle_embedded_file("index.html")
}

fn dist(req: web::HttpRequest) -> HttpResponse {
    let path = &req.path()["/".len()..];
    handle_embedded_file(path)
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
