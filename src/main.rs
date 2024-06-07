#![warn(clippy::str_to_string)]
mod commands;
use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use ::serenity::all::{Attachment, AttachmentType, CreateAttachment, CreateButton, CreateMessage, Interaction, Message, ReactionType};
use std::{
    collections::HashMap, fmt, io::Cursor, sync::{Arc, Mutex}, time::Duration
};
use anyhow::{bail, Result};
use thiserror::Error;
use reqwest;
use image::{DynamicImage, load_from_memory, ImageFormat};
use tokio::{io::AsyncWriteExt, runtime::Runtime};

// Types used by all command functions
type AsyncError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, AsyncError>;
use log::{info, warn, debug, Level::Debug, set_max_level};
use failure::{Backtrace, Fail};

mod colors;

// Custom user data passed to all command functions
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, AsyncError>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn interaction_create(&self, ctx: serenity::Context, interaction: Interaction) {
    if let Interaction::Component(interaction) = interaction {
        let content = match interaction.data.custom_id.as_str() {
            "button_1" => "You clicked Button 1",
            "button_2" => "You clicked Button 2",
            _ => "Unknown button",
        };
        // let builder = CreateMessage::new().
        // interaction.create_response(&ctx.http, ).await.unwrap();
    }
}


#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![commands::help(), commands::vote(), commands::getvotes()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot"),
                poise::Prefix::Literal("hey bot,"),
            ],
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        // command_check: Some(|ctx| {
        //     Box::pin(async move {
        //         if ctx.author().id == 123456789 {
        //             return Ok(false);
        //         }
        //         Ok(true)
        //     })
        // }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();

    // let token = var("DISCORD_TOKEN")
    //     .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set in .env");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, AsyncError>,
    data: &Data,
) -> Result<(), AsyncError> {
    println!(
        "Got an event in event handler: {:?}",
        event.snake_case_name()
    );

    // if let FullEvent::Message(msg) = event {
    //     handle_message_create(&msg).await?;
    // }

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            for attachment in &new_message.attachments {
                if new_message.author.bot {
                    continue;
                }
                println!("attachment found");
                println!(
                    "media type: {:?}; filename: {}; Size: {} MiB; URL: {}", 
                    attachment.content_type, attachment.filename, attachment.size as f64 / 1024.0 / 1024.0, attachment.url
                );
                let image = process_image(&attachment, &message).await.unwrap();
                let mut buffer = Vec::new();
                image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png).unwrap();
                // Sends an embed with a link to the original image ~~and the prided image attached~~.\
                let attachment = CreateAttachment::bytes(buffer, "image.png");
                let content = CreateMessage::default().add_file(attachment).content("dark image");
                new_message.channel_id
                    .send_message(ctx, content)
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn process_image(attachment: &serenity::Attachment, msg: &Message) -> Result<DynamicImage> {
    let mib = attachment.size as f64 / 1024.0 / 1024.0;
    if mib > 16.0 {
        bail!("File too large: {} MiB", mib);
    }
    if attachment.content_type.is_none() {
        bail!("No content type found for attachment");
    }
    let content_type = attachment.content_type.as_ref().unwrap();
    if !content_type.starts_with("image/") {
        bail!("Attachment is not an image: {}", content_type);
    }
    let url = attachment.url.clone();
    let mut image = download_image(&attachment).await?;
    let bright = colors::calculate_average_brightness(&image.to_rgba8());
    if bright > 0.4 {
        bail!("Not bright enough: {bright}")
    }
    let response = CreateMessage::new()
        .content("Bruhh...\n\nThis looks bright as fuck. May I darken it?")
        .button(CreateButton::new(format!("darken-{}", )))
    // check brightness, ask user
    Ok(colors::apply_nord(image))
}

async fn download_image(attachment: &Attachment) -> Result<DynamicImage> {
    // Send the GET request
    println!("Downloading: {}=&format=png", attachment.proxy_url);
    let response = reqwest::get(format!("{}=&format=png", attachment.proxy_url)).await?;
    
    // Ensure the request was successful
    if !response.status().is_success() {
        info!("Request failed with status code: {}", response.status());
        anyhow::bail!("Request failed with status code: {}", response.status());
    }
   
    let bytes = response.bytes().await?;
    // let raw = attachment.download().await?;
    // Get the image bytes
    println!("Downloaded image with {} bytes", bytes.len());
    // Load the image from the bytes
    let image = image::load_from_memory(&bytes).map_err(
        |e| anyhow::anyhow!("Failed to load image: {}", e)
    )?;
    
    Ok(image)
}