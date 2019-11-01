mod config;
use crate::config::{Config, read_config};


mod tweets;
pub(crate) use tweets::{produce_tweets, deserialize_tweets, Tweet};


mod prelude;
use crate::prelude::*;


struct Sentiment {
	pub score: f32,
	pub keyword: &'static str,
}

// We know that there are a small and finite set of keywords. Leaking them
// allows us to copy the references around rather than using
// Arc<String> or similar for better performance
fn leak_keywords(mut keywords: Vec<String>) -> Vec<&'static str> {
	keywords
		.drain(..)
		.map(|k| Box::leak::<'static>(Box::new(k)).as_str())
		.collect()
}

async fn async_main() -> Result<()> {
	let config = read_config().await?;
	let Config { keywords, auth } = config;
	let keywords = leak_keywords(keywords);
	let raw_tweets = produce_tweets(auth, keywords.clone());
	let (tweets_sender, mut tweets) = unbounded();
	let t = task::spawn(deserialize_tweets(raw_tweets, tweets_sender)); // TODO: Multi-thread deserialization

	loop {
		let next = tweets.next().await;
		let next = match next {
			None => break,
			Some(n) => n,
		};
		let Tweet { text } = next;
		let text = if let Some(text) = text { text } else { continue; };
		println!("{}", text);
	}
	dbg!("End");

	Ok(())
}

fn main() -> Result<()> {
	task::block_on(async_main())?;
	Ok(())
}
