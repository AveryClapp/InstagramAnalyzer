use dotenv::dotenv;
use tokio;
use rusqlite::{Connection, Result as SqliteResult};
use std::env;
use thirtyfour::prelude::*;
use std::time::Duration;
use std::collections::HashSet;


// Define the InstaaGram Scraper struct
struct InstagramScraper {
    username: String,
    password: String,
    db_conn: Connection,
}


#[tokio::main]
// Declare asynchronous main function. The return value (Result, is either a success message or an error)
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Load the dotenv file from the current directory
    dotenv().ok();

    //Get Username
    let insta_user = env::var("username").expect("Password must be set in the .env file");

    //Get Password
    let insta_pass = env::var("password").expect("Username must be set in the .env file");

    // Get path to DB
    let db_path_str = format!("./{}_data.db", insta_user);

    // Create a new instance of the Instagram Analyzer object, the ? will return early from the function if there is an error
    let analyzer = InstagramScraper::new(insta_user, insta_pass, db_path_str).await?;

    // Initialize DB tables if they do not exist
    analyzer.init_db()?;

    analyzer.navigate().await?; 

    // Return as a success
    Ok(())
}

// Create the Instagram Analyzer struct
impl InstagramScraper {
    async fn new(username: String, password: String, db_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let db_conn = Connection::open(db_url)?;

        Ok(InstagramScraper {
            username,
            password,
            db_conn,
        })
    }

    async fn navigate(&self) -> Result<(), WebDriverError> {
        let mut caps = DesiredCapabilities::chrome();
        // caps.add_chrome_arg("--headless")?;
        caps.add_chrome_arg("--disable-gpu")?;
        let driver = WebDriver::new("http://localhost:9515", caps).await?;

        driver.goto("https://www.instagram.com").await?;

        let username_element = driver.find(By::Name("username")).await?;
        username_element.wait_until().displayed().await?;
        username_element.send_keys(&self.username).await?;

        let password_element = driver.find(By::Name("password")).await?;
        password_element.wait_until().displayed().await?;
        password_element.send_keys(&self.password).await?;

        let submit_button = driver.find(By::XPath("//button[@type='submit']")).await?;
        submit_button.wait_until().displayed().await?;
        submit_button.click().await?;
        
        //Log in to instagram
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Navigate to page of followers
        driver.goto(format!("https://www.instagram.com/{}/followers/", &self.username)).await?;
        
        let mut users = HashSet::new();
        while users.len() < 1000 {
            let followers = driver.find_all(By::XPath("//a[contains(@href, '/')]")).await?;
    
            for follower in followers {
                if let Ok(Some(href)) = follower.attr("href").await {
                    let username = href.split("/").nth(3).unwrap_or("").to_string();
                    if !username.is_empty() {
                        users.insert(username);
                    }
                }
            }
    
            driver.execute(
                "window.scrollTo(0, document.body.scrollHeight)",
                vec![]
            ).await?;
    
            // Wait for content to load
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Ok(())
        
    }

    fn init_db(&self) -> SqliteResult<()> {
        self.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS macro_data (
                id TEXT PRIMARY KEY,
                caption TEXT,
                media_type TEXT,
                timestamp INTEGER,
                like_count INTEGER,
                comments_count INTEGER
            )",
            [],
        )?;
        self.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS follower_data (
                username TEXT PRIMARY KEY
            )",
            [],
        )?;
        self.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS following_data (
                username TEXT PRIMARY KEY
            )",
            [],
        )?;
        Ok(())
    }
}