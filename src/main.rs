#![feature(proc_macro_hygiene, decl_macro)]

use async_std::stream::interval;
use core::time::Duration;
use futures::stream::select;
use std::sync::Arc;
use std::sync::RwLock;

const UPDATE_INTERVAL_SECONDS: u64 = 4;

mod config;
use crate::config::{read_config, Config};

mod tweets;
use tweets::produce_tweets;

mod prelude;
use crate::prelude::*;

mod sentiment_pipeline;
use sentiment_pipeline::{get_sentiments_from_tweets, Sentiment};

mod aggregator;
use aggregator::Aggregator;

mod webserver;
use webserver::launch;

// We know that there are a small and finite set of keywords. Leaking them
// allows us to copy the references around rather than using
// Arc<String> or similar for better performance
fn leak_keywords(mut keywords: Vec<String>) -> Keywords {
    let keywords: Vec<_> = keywords
        .drain(..)
        .map(|k| Box::leak::<'static>(Box::new(k)).as_str())
        .collect();
    Box::leak(Box::new(keywords))
}

async fn async_main() -> Result<()> {
    // Config
    let Config { keywords, auth } = read_config().await?;
    let keywords = leak_keywords(keywords);

    // Get a channel of keyword matches with the sentiment calculated for the tweet.
    // Utilizes multiple long running subtasks.
    let raw_tweets = produce_tweets(auth, keywords);
    let (sentiments_send, sentiments) = unbounded();
    task::spawn(get_sentiments_from_tweets(
        raw_tweets,
        sentiments_send,
        keywords,
    ));

    // Our results will be piped into here.
    let mut aggregator = Aggregator::new(keywords);
    aggregator.sample(); // Hack to ensure we don't need to handle empty lists on the browser side.

    let shared = Arc::new(RwLock::new(Arc::new(aggregator.clone())));

    // Now that we have data, even if it's empty, we can start the webserver.
    launch(shared.clone());

    let mut events = select(
        interval(Duration::from_secs(UPDATE_INTERVAL_SECONDS)).map(|_| UpdateOrSentiment::Update),
        sentiments.map(UpdateOrSentiment::Sentiment),
    );

    loop {
        match events.next().await {
            None => break,
            Some(UpdateOrSentiment::Update) => {
                aggregator.sample();
                let clone = Arc::new(aggregator.clone());
                let mut w = shared.write().unwrap();
                *w = clone;
            }
            Some(UpdateOrSentiment::Sentiment(sentiment)) => {
                aggregator.add(sentiment);
            }
        };
    }

    Ok(())
}

enum UpdateOrSentiment {
    Update,
    Sentiment(Sentiment),
}

fn main() -> Result<()> {
    task::block_on(async_main())?;
    Ok(())
}
