use async_std::stream::interval;
use core::time::Duration;
use futures::stream::select;

const UpdateIntervalSeconds: u64 = 120;

mod config;
use crate::config::{Config, read_config};

mod tweets;
use tweets::produce_tweets;

mod prelude;
use crate::prelude::*;

mod sentiment_pipeline;
use sentiment_pipeline::{get_sentiments_from_tweets, Sentiment};

mod aggregator;
use aggregator::Aggregator;

enum UpdateOrSentiment {
	Update,
	Sentiment(Sentiment),
}

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
	let config = read_config().await?;
	let Config { keywords, auth } = config;
	let keywords = leak_keywords(keywords);
	let raw_tweets = produce_tweets(auth, keywords);
	let (sentiments_send, sentiments) = unbounded();
	task::spawn(get_sentiments_from_tweets(raw_tweets, sentiments_send, keywords));

	let mut aggregator = Aggregator::new(keywords);
	aggregator.sample(); // Ensure we don't need to handle empty lists

	let mut events = select(
		interval(Duration::from_secs(UpdateIntervalSeconds)).map(|_| UpdateOrSentiment::Update),
		sentiments.map(UpdateOrSentiment::Sentiment)
	);

	loop {
		match events.next().await {
			None => break,
			Some(UpdateOrSentiment::Update) => {
				aggregator.sample();
				dbg!(&aggregator);
			},
			Some(UpdateOrSentiment::Sentiment(sentiment)) => {
				aggregator.add(sentiment);
			}
		};

	}

	Ok(())
}

fn main() -> Result<()> {
	task::block_on(async_main())?;
	Ok(())
}
