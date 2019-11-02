use crate::prelude::*;
use crate::tweets::Tweet;
use second_stack::acquire;
use sentiment::analyze;
use serde_json;

pub struct Sentiment {
    pub score: f32,
    pub keyword: &'static str,
}

// TODO: Error handling

async fn get_sentiment_from_tweet(raw: String, out: Sender<Sentiment>, keywords: Keywords) {
    let tweet: serde_json::Result<Tweet> = serde_json::from_str(&raw);

    // If there is any problem deserializing, or there is no text at all, just drop the tweet.
    let text = if let Ok(Tweet { text: Some(text) }) = tweet {
        text
    } else {
        return;
    };

    // Find the keywords first, since the sentiment analysis consumes the string
    // Since this is a test and there is a code review which considers performance,
    // look at this crate I made called second-stack that avoids an allocation of a Vec here.
    // TODO: Unicase and tokenization to ensure we aren't matching substrings of other words, (unless twitter does this),
    // and that we're getting all the varieties of words.
    let matching_keywords = acquire(keywords.iter().filter(|k| text.contains(*k)));
    // It is not too uncommon for empty, because often it's something other than the text which contains the keyword
    if matching_keywords.is_empty() {
        return;
    }

    let sentiment = analyze(text);

    for matching_keyword in matching_keywords.iter() {
        let result = Sentiment {
            score: sentiment.score,
            keyword: matching_keyword,
        };
        out.unbounded_send(result).unwrap();
    }
}

pub async fn get_sentiments_from_tweets(
    mut tweets: impl Stream<Item = String> + Unpin,
    sentiments: Sender<Sentiment>,
    keywords: Keywords,
) {
    loop {
        let raw = match tweets.next().await {
            None => break,
            Some(n) => n,
        };

        let send = sentiments.clone();
        task::spawn(get_sentiment_from_tweet(raw, send, keywords));
    }
}
