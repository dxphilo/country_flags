use dotenvy::dotenv;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::var;
use std::{fs, path::Path};
use tokio::time::{sleep, Duration};
use tweety_rs::{
    types::tweet::{Media, PostTweetParams},
    TweetyClient,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryBot {
    pub rand_number: usize,
    pub file_content: (String, String),
}

impl CountryBot {
    pub async fn new(path: &Path) -> Self {
        let file = fs::read_to_string(path).unwrap();

        let json_content = serde_json::from_str(&file).unwrap();
        if let Value::Object(map) = json_content {
            let key_value_pairs = map
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.as_str().unwrap_or("").to_string()))
                .collect::<Vec<(String, String)>>();

            let random_index = rand::thread_rng().gen_range(0..key_value_pairs.len());

            CountryBot {
                rand_number: random_index,
                file_content: key_value_pairs[random_index].clone(),
            }
        } else {
            println!("json content is not an object");
            CountryBot {
                rand_number: 0,
                file_content: ("KE".to_owned(), "Kenya".to_owned()),
            }
        }
    }
    pub async fn post_to_twitter(
        self,
        consumer_key: &str,
        consumer_secret: &str,
        access_token: &str,
        access_secret: &str,
    ) {
        let rand_country_key = self.file_content.0;
        let file_path = format!("flags/{}.png", rand_country_key);

        let client = TweetyClient::new(
            &consumer_key,
            &access_token,
            &consumer_secret,
            &access_secret,
        );

        let path = Path::new(&file_path);

        match client.upload_file(&path).await {
            Ok(value) => {
                let media_string = value.to_string();
                let message = format!("#{}", self.file_content.1);

                let params = PostTweetParams {
                    direct_message_deep_link: None,
                    for_super_followers_only: None,
                    geo: None,
                    media: Some(Media {
                        media_ids: vec![media_string].into(),
                        tagged_user_ids: None,
                    }),
                    poll: None,
                    quote_tweet_id: None,
                    reply: None,
                    reply_settings: None,
                };

                match client.post_tweet(&message, Some(params)).await {
                    Ok(status_code) => {
                        println!("Posted tweet with status code: {:?}", status_code);
                    }
                    Err(err) => {
                        println!("Error posting tweet: {}", err);
                    }
                }
            }
            Err(err) => {
                println!("Error uploading images{}", err);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let access_token = var("ACCESS_TOKEN").unwrap();
    let access_secret = var("ACCESS_TOKEN_SECRET").unwrap();
    let consumer_key = var("CONSUMER_API_KEY").unwrap();
    let consumer_secret = var("CONSUMER_API_SECRET").unwrap();

    let json_path = Path::new("countries.json");

    loop {
        CountryBot::new(json_path)
            .await
            .post_to_twitter(
                &consumer_key,
                &consumer_secret,
                &access_token,
                &access_secret,
            )
            .await;
        sleep(Duration::from_secs(1800)).await; // Wait for 30 minutes
    }
}
