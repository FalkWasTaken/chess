#[macro_use]
extern crate rocket;

#[macro_use]
extern crate lazy_static;

use std::time::Instant;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::serde::json::Json;
use serde::Serialize;

mod types;
mod engines;
mod utils;
mod game;
mod score_functions;
mod byte_board;

use crate::engines::*;
use types::*;
use utils::pos_to_string;

#[derive(Serialize, Debug)]
pub struct MoveResponse {
    pub from: String,
    pub to: String,
    pub score: Score,
    pub checkmate: bool
}

impl MoveResponse {
    fn new(mv: Option<Move>, score: Score) -> MoveResponse {
        let mut res = match mv {
            Some((from, to)) => MoveResponse {
                from: pos_to_string(from), 
                to: pos_to_string(to), 
                score,
                checkmate: false
            },
            None => MoveResponse {
                from: String::new(), 
                to: String::new(), 
                score,
                checkmate: true
            }
        };
        if score == Score::INFINITY {
            res.score = 100.0;
        } else if score == Score::NEG_INFINITY {
            res.score = -100.0;
        }
        res
    }
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


#[get("/make_move?<fen>")]
fn make_move(fen: &str) -> Result<Json<MoveResponse>, BadRequest> {
    let init = Instant::now();
    let mut engine = First::new(fen, 6)?;
    let (mv, score) = engine.make_move();
    let res = MoveResponse::new(mv, score);
    println!("Visited leafs: {}", engine.num_leafs);
    //println!("Score = {score}, response = {:?}", res);
    println!("Elapsed time: {}ms", init.elapsed().as_millis());
    Ok(Json(res))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/engine", routes![make_move])
        .attach(CORS)
}