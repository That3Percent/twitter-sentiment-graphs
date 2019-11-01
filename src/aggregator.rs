use chrono::{DateTime, Utc};
use std::collections::HashMap;
use circular_queue::CircularQueue;
use crate::prelude::*;
use crate::sentiment_pipeline::Sentiment;

const SAMPLES: usize = 100;

#[derive(Debug)]
pub struct Aggregator {
	items: HashMap<&'static str, AggregatorItem>,
	sample_times: CircularQueue<DateTime<Utc>>,
}

impl Aggregator {
	pub fn new(keywords: Keywords) -> Self {
		Aggregator {
			items: keywords.iter().map(|k| (*k, AggregatorItem::new())).collect(),
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
			let average = if item.current_count == 0 {
				// Considering no sentiment as neutral sentiment. No sentiment could be represented differently,
				// but choosing not to for simplicity.
				0.
			} else {
				let average = item.current_total_sentiment / (item.current_count as f32);
				item.current_total_sentiment = 0.;
				item.current_count = 0;
				average
			};
			item.samples.push(average);
		}
	}
}

#[derive(Debug)]
struct AggregatorItem {
	current_count: usize,
	current_total_sentiment: f32,
	samples: CircularQueue<f32>,
}

impl AggregatorItem {
	pub fn new() -> Self {
		AggregatorItem {
			current_count: 0,
			current_total_sentiment: 0.,
			samples: CircularQueue::with_capacity(SAMPLES),
		}
	}
}