use crate::prelude::*;
use crate::tweets::Tweet;
use second_stack::{acquire, StackAlloc};
use sentiment::analyze;
use serde_json;
use unicase::UniCase;

pub struct Sentiment {
    pub score: f32,
    pub keyword: &'static str,
}

fn find_matching_keywords(keywords: Keywords, text: &str) -> StackAlloc<bool> {
    // TODO: This can be simplified if I add either reserve() or filter() to second-stack
    let mut matching_keywords = acquire(std::iter::repeat(false).take(keywords.len()));

    let words = text.split(|c: char| !c.is_alphabetic());

    for word in words {
        for (index, keyword) in keywords.iter().enumerate() {
            // FIXME: This is not correct unicode handling to verify the length here,
            // but there is a bug in the UniCase crate.
            // I filed an issue here: https://github.com/seanmonstar/unicase/issues/38
            // This hack should (probably?) be ok for the test, since all the keywords given were ascii.
            if word.len() == keyword.len() && UniCase::unicode(word) == UniCase::unicode(keyword) {
                matching_keywords[index] = true;
            }
        }
    }

    matching_keywords
}

async fn get_sentiment_from_tweet(raw: String, out: Sender<Sentiment>, keywords: Keywords) {
    let tweet: serde_json::Result<Tweet> = serde_json::from_str(&raw);

    // If there is any problem deserializing, or there is no text at all, just drop the tweet.
    // TODO: Consider panic on failure to deserialize. This likely means that the twitter format
    // has changed, and the program needs to be updated. Otherwise, the program will appear to
    // be running successfully but gather no data.
    let text = if let Ok(Tweet { text: Some(text) }) = tweet {
        text
    } else {
        return;
    };

    // Find the keywords first, since the sentiment analysis consumes the string
    let matching_keywords = find_matching_keywords(keywords, &text);
    // It is not too uncommon for empty, because often it's something other than the text which contains the keyword
    if !matching_keywords.iter().any(|k| *k) {
        return;
    }

    let sentiment = analyze(text);

    for (index, is_match) in matching_keywords.iter().enumerate() {
        if !is_match {
            continue;
        }
        let result = Sentiment {
            score: sentiment.score,
            keyword: keywords[index],
        };
        out.unbounded_send(result).unwrap();
    }
}

/// Takes a raw tweet (json serialized) and does the following...
/// * Deserialize
/// * Find matching keywords
/// * Calculate sentiment
/// * Publish sentiment for all matching keywords on the provided channel.
///
/// Silently ignores errors
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

        // Choosing the individual tweet as the largest unit of work for a task.
        // Breaking it down further would add complexity for no additional throughput.
        task::spawn(get_sentiment_from_tweet(raw, send, keywords));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_match_sub_words() {
        // This fails because of a problem with a dependency.
        // I filed an issue here: https://github.com/seanmonstar/unicase/issues/38
        let text = "Everyone loves pizza";
        let keywords = &["love"];
        let matches = find_matching_keywords(keywords, text);
        assert_eq!(matches[0], false);
    }

    #[test]
    fn matches_multiple_words() {
        let text = "I love pizza";
        let keywords = &["pizza", "bob", "love"];
        let matches = find_matching_keywords(keywords, text);
        assert_eq!(&matches[..], &[true, false, true]);
    }

    #[test]
    fn match_is_case_insensitive() {
        let text = "I!LOVE!PIZZA!";
        let keywords = &["pizza"];
        let matches = find_matching_keywords(keywords, text);
        assert_eq!(matches[0], true);
    }

    #[test]
    fn does_not_match_nonsense() {
        let text = "RT @zunigasphotos: Recently my nose has been bleeding a lot and eloy surprised me with a spooky basket and this happened ����😂😂😂😂 https://t.co/…";
        let keywords = &["love"];
        let matches = find_matching_keywords(keywords, text);
        assert_eq!(matches[0], false);
    }
}
