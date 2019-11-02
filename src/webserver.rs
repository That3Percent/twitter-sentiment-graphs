use crate::aggregator::Aggregator;
use crate::prelude::*;
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
fn data(state: State<Snapshot>) -> Json<Data> {
    // Don't hold the lock for very long.
    let lock = state.read().unwrap();
    let agg = lock.clone();
    drop(lock);

    // Convert to a json ready structure. If a lot of concurrent
    // client connections are expected, we would split this and
    // the conversion to json binary into a snapshotting task and
    // this would only memcopy to the stream.
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

/// Starts the webserver in it's own long running thread.
/// It manages tasks independently of the analysis.
pub fn launch(data: Arc<RwLock<Arc<Aggregator>>>) {
    std::thread::spawn(move || {
        rocket::ignite()
            .manage(data)
            .mount("/", routes![index, data,])
            .launch();
    });
}
