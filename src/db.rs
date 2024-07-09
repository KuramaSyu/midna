use image::{load_from_memory, DynamicImage};
use sha2::{Sha256, Digest};
use tokio::io::AsyncReadExt;
use std::fs::File;
use std::io::{self, Read};
use hex;
use tokio_postgres::{Error, NoTls, Row};
use anyhow::{bail, Result};



fn calculate_sha256(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    Ok(hex::encode(hasher.finalize()))
}

// connects to the database and executes the setup.sql
pub async fn connect() -> Result<tokio_postgres::Client> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=your_password dbname=your_db", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to the database
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=your_password dbname=your_db", NoTls).await?;

    // Spawn a new task to run the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Path to your image file
    let file_path = "path/to/your/image.png";

    // Calculate the SHA-256 hash
    let hash = calculate_sha256(file_path)?;

    // Read the image file
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut image_data = Vec::new();
    file.read(&mut image_data).await?;

    // Insert the image and its hash into the database
    client.execute(
        "INSERT INTO images (hash, image) VALUES ($1, $2)",
        &[&hash, &image_data],
    ).await?;

    println!("Image stored with hash: {}", hash);

    Ok(())
}

pub async fn get_image_from_db(hash: &str) -> Result<DynamicImage> {
    // Connect to the database
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=your_password dbname=your_db", NoTls).await?;

    // Spawn a new task to run the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Query the database for the image data
    let row: Row = client.query_one(
        "SELECT image FROM images WHERE hash = $1",
        &[&hash],
    ).await?;

    // Get the image data as a byte array
    let image_data: Vec<u8> = row.get(0);

    // Convert the byte array into a DynamicImage
    let dynamic_image = load_from_memory(&image_data)?;

    Ok(dynamic_image)
}