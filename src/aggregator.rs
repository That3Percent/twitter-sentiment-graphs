use crate::prelude::*;
use crate::sentiment_pipeline::Sentiment;
use chrono::{DateTime, Utc};
use circular_queue::CircularQueue;
use std::collections::HashMap;

const SAMPLES: usize = 65;

/// Collects averages of sentiment for a collection of keywords over time.
#[derive(Debug, Clone)]
pub struct Aggregator {
    pub items: HashMap<&'static str, AggregatorItem>,
    pub sample_times: CircularQueue<DateTime<Utc>>,
}

impl Aggregator {
    pub fn new(keywords: Keywords) -> Self {
        Aggregator {
            items: keywords
                .iter()
                .map(|k| (*k, AggregatorItem::new()))
                .collect(),
            sample_times: CircularQueue::with_capacity(SAMPLES),
        }
    }

    pub fn add(&mut self, sentiment: Sentiment) {
        let Sentiment { score, keyword } = sentiment;
        let collection = self.items.get_mut(keyword).unwrap();
        collection.current_total_sentiment += score;
        collection.current_count += 1;
    }

    pub fn sample(&mut self) {
        let now = Utc::now();
        self.sample_times.push(now);
        for item in self.items.values_mut() {
            item.move_current_to_history();
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregatorItem {
    current_count: usize,
    current_total_sentiment: f32,
    pub samples: CircularQueue<f32>,
}

impl AggregatorItem {
    pub fn new() -> Self {
        AggregatorItem {
            current_count: 0,
            current_total_sentiment: 0.,
            samples: CircularQueue::with_capacity(SAMPLES),
        }
    }

    /// The average sentiment for tweets in the latest sample which
    /// has not yet been moved to history.
    fn current(&self) -> f32 {
        if self.current_count == 0 {
            // Considering no sentiment as neutral sentiment. No sentiment could be represented differently,
            // but choosing not to for simplicity.
            0.
        } else {
            self.current_total_sentiment / (self.current_count as f32)
        }
    }

    /// Moves the current sentiment into the samples history
    fn move_current_to_history(&mut self) {
        self.samples.push(self.current());
        self.current_total_sentiment = 0.;
        self.current_count = 0;
    }
}
