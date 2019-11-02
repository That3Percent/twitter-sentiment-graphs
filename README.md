# Purpose
This project monitors a live stream of twitter keywords, analyzes them for sentiment, and
serves the results via a live updating graph. It was created for a coding test, so
if you want production value look elsewhere.

# Setup Instructions
 * Clone the repository
 * Copy the json provided below into config.json in the root directory of the project
 * Add your twitter credentials to config.json
 * Set your default compiler to nightly. "rustup default nightly"
 * cargo run --release
 * visit http://localhost:8000

## Config
```
{
	"keywords": ["twitter", "facebook", "google", "travel", "art", "music", "photography", "love", "fashion", "food"],
	"auth": {
		"consumerKey": "",
		"consumerSecret": "",
		"accessKey": "",
		"accessSecret": ""
	}
}
```

# Architecture
![Architecture](https://github.com/That3Percent/twitter-sentiment-graphs/blob/master/Architecture.png)