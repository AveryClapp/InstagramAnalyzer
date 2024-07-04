use dotenv::dotenv;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Row};
use std::env;

#[tokio::main]
// Declare asynchronous main function. The return value (Result, is either a success message or an error)
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Load the dotenv file from the current directory
    dotenv().ok();

    // Get the access token of the user from the .env, throw an error if it doesn't exist
    let access_token = env::var("INSTAGRAM_KEY").expect("INSTAGRAM_KEY must be set with the .env file");
    
    // Setup database connection, will create file if it does not exist in your directory
    let db_url = "sqlite:instagram_data.db"

    // Create a new instance of the Instagram Analyzer object, the ? will return early from the function if there is an error
    let analyer = InstagramAnalyzer::new(access_token, db_url).await?;

    // This is an async function so we must use the .await for error handling
    let posts = analyzer.get_data_posts().await?;

    // Store posts and pass the reference so we don't move post data in memory
    analyzer.store_posts(&posts).await?;

    // Calculate engagement rate from analyzer's method
    let engagement_rate = analyzer.calculate_engagement_rate().await?;

    // Return as a success
    Ok(())
}

// Create the Instagram Analyzer struct
impl InstagramAnalyzer {
    //Constructor
    async fn new(access_token: String, db_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Create HTTP Client
        let client = reqwest::Client::new();

        // Setup DB connection to db_url with 5 max connections
        let db_pool = SqlitePoolOptions::new().max_connections(5).connect(db_url).await?;

        Ok(InstagramAnalyzer{
            client,
            access_token,
            db_pool
        })
    }
}