use serde::{Deserialize};
use crate::prelude::*;

#[derive(Deserialize)]
pub struct Tweet {
	pub text: Option<String>,
}

#[derive(Deserialize)]
pub struct TwitterAuth {
	#[serde(rename="consumerKey")]
	pub consumer_key: String,
	#[serde(rename="consumerSecret")]
	pub consumer_secret: String,
	#[serde(rename="accessKey")]
	pub access_key: String,
	#[serde(rename="accessSecret")]
	pub access_secret: String,
}


pub async fn deserialize_tweets(mut tweets: impl Stream<Item=String> + Unpin, mut out: Sender<Tweet>) {
	while let Some(tweet) = tweets.next().await {
		let tweet: serde_json::Result<Tweet> = serde_json::from_str(&tweet);
		out.unbounded_send(tweet.unwrap()); // TODO: Error handling
	}
}

pub fn produce_tweets(auth: TwitterAuth, keywords: Vec<&'static str>) -> impl Stream<Item=String> {
	let (send, recv) = unbounded();

	std::thread::spawn(move || {
		use twitter_stream::{Token, TwitterStreamBuilder};
		use twitter_stream::rt::{self, Future, Stream};

		let TwitterAuth {
			consumer_key,
			consumer_secret,
			access_key,
			access_secret
		} = auth;

		let token = Token::new(consumer_key, consumer_secret, access_key, access_secret);

		let track = itertools::join(keywords, ",");
		let fut = TwitterStreamBuilder::filter(token)
			.track(Some(track.as_str()))
			.listen()
			.unwrap()
			.flatten_stream()
			.for_each(move |json| {
				send.unbounded_send(json.to_string()).unwrap(); // TODO: Error handling
				Ok(())
			}).map_err(|e| {dbg!(e); ()}); // TODO: Handle reconnection

		rt::run(fut);
	});

	recv
}

// TODO: We wanted the much more straightforward version like so...
// However, Old/New tokio didn't play well and the stream just times out immediately.
// The current solution does have the advantage of
/*

pub fn produce_tweets() -> impl futures::Stream<Item=Result<String, Error>> {
	use crossbeam_channel::Sender;
	use twitter_stream::{Token, TwitterStreamBuilder, Error};
	use twitter_stream::rt::{Future, Stream};
	use futures::compat::Stream01CompatExt;


    let token = Token::new(ConsumerKey, ConsumerSecret, AccessKey, AccessSecret);

	let track = itertools::join(Keywords, "&");

    TwitterStreamBuilder::filter(token)
        .track(Some(track.as_str()))
        .listen()
	.unwrap()
        .flatten_stream()
		.map(|e| e.to_string() )
		.compat()
}
*/