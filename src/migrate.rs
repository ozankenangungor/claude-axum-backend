use todo_api::db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from env
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    
    println!("Connecting to database...");
    let db_pool = db::connection_pool(&database_url).await?;
    
    println!("Running migrations...");
    db::schema::initialize_schema(&db_pool).await?;
    
    println!("✅ All migrations completed successfully!");
    
    Ok(())
}