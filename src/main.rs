use dotenv::dotenv;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::env;
use std::path::PathBuf;


// Define the Post struct
#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: String,
    caption: Option<String>,
    media_type: String,
    timestamp: DateTime<Utc>,
    like_count: i32,
    comments_count: i32,
}

// Define the InstagramAnalyzer struct
struct InstagramAnalyzer {
    client: reqwest::Client,
    access_token: String,
    db_conn: Connection,
}


#[tokio::main]
// Declare asynchronous main function. The return value (Result, is either a success message or an error)
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Load the dotenv file from the current directory
    dotenv().ok();

    // Get the access token of the user from the .env, throw an error if it doesn't exist
    let access_token = env::var("INSTAGRAM_KEY").expect("INSTAGRAM_KEY must be set with the .env file");
    
    // Get path to DB
    let db_path_str = "./instagram_analyzer.db";

    // Create a new instance of the Instagram Analyzer object, the ? will return early from the function if there is an error
    let analyzer = InstagramAnalyzer::new(access_token, db_path_str).await?;

    // Initialize DB if it does not exist
    analyzer.init_db()?;

    // This is an async function so we must use the .await for error handling
    let posts = analyzer.get_user_posts().await?;

    // Store posts and pass the reference so we don't move post data in memory
    analyzer.store_posts(&posts).await?;

    // Calculate engagement rate from analyzer's method
    let engagement_rate = analyzer.calculate_engagement_rate().await?;
    println!("Engagement Rate is: {:.2}", engagement_rate);
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
        let db_conn = Connection::open(db_url)?;

        Ok(InstagramAnalyzer{
            client,
            access_token,
            db_conn,
        })
      
    }
    
    // Fetch user posts from Instagram
    async fn get_user_posts(&self) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        // Fetch the data from the 'me/media' endpoint
        let data = self.fetch_data("me/media").await?;

        // Parse the JSON data into a Vec<Post>
        let posts: Vec<Post> = serde_json::from_value(data["data"].clone())?;

        Ok(posts)
    }

    // Fetch data from the Instagram API
    async fn fetch_data(&self, endpoint: &str) -> Result<serde_json::Value, reqwest::Error> {
        // Construct the full URL for the API request/
        let url = format!("https://graph.instagram.com/{}", endpoint);

        // Send a GET request to the API with the access token in the query parameters
        // Then parse the response as JSON
        let response = self.client
            .get(&url)
            .query(&[("access_token", &self.access_token)])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    // Store posts in the database
    async fn store_posts(&self, posts: &[Post]) -> Result<(), Box<dyn std::error::Error>> {
        for post in posts {
            self.db_conn.execute(
                "INSERT OR REPLACE INTO posts (id, caption, media_type, timestamp, like_count, comments_count) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &post.id,
                    &post.caption,
                    &post.media_type,
                    post.timestamp.timestamp(),
                    post.like_count,
                    post.comments_count
                ],
            )?;
        }
        Ok(())
    }

    async fn calculate_engagement_rate(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let mut stmt = self.db_conn.prepare(
            "SELECT AVG(like_count + comments_count) as avg_engagement FROM posts"
        )?;
        let avg_engagement: f64 = stmt.query_row([], |row| row.get(0))?;
        Ok(avg_engagement)
    }

    fn init_db(&self) -> SqliteResult<()> {
        self.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS posts (
                id TEXT PRIMARY KEY,
                caption TEXT,
                media_type TEXT,
                timestamp INTEGER,
                like_count INTEGER,
                comments_count INTEGER
            )",
            [],
        )?;
        Ok(())
    }
}