use crate::aggregator::Aggregator;
use arc_swap::ArcSwap;
use rocket::response::NamedFile;
use rocket::{self, get, routes, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use std::sync::{Arc, RwLock};

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open("index.html").ok()
}

#[get("/data")]
fn data(state: State<Arc<RwLock<Arc<Aggregator>>>>) -> Json<Data> {
    let lock = state.read().unwrap();
    let agg = lock.clone();
    drop(lock);
    let data = Data {
        times: agg
            .sample_times
            .iter()
            .map(|t| t.timestamp() as f64 * 1000.)
            .collect(),
        keywords: agg
            .items
            .iter()
            .map(|(k, v)| Keyword {
                name: k,
                scores: v.samples.iter().map(|f| *f as f64).collect(),
            })
            .collect(),
    };
    Json(data)
}

#[derive(Serialize)]
struct Keyword {
    name: &'static str,
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct Data {
    times: Vec<f64>,
    keywords: Vec<Keyword>,
}

pub fn launch(data: Arc<RwLock<Arc<Aggregator>>>) {
    std::thread::spawn(move || {
        rocket::ignite()
            .manage(data)
            .mount("/", routes![index, data,])
            .launch();
    });
}
