#![warn(clippy::str_to_string)]
mod commands;
use colors::NordOptions;
use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use ::serenity::all::{Attachment, AttachmentType, ButtonStyle, ComponentInteraction, CreateActionRow, CreateAttachment, CreateButton, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, EditAttachments, EditInteractionResponse, Interaction, Message, ReactionType};
use std::{
    collections::HashMap, fmt, io::Cursor, sync::{Arc, Mutex}, time::Duration
};
use anyhow::{bail, Result};
use thiserror::Error;
use reqwest;
use image::{codecs::png::{CompressionType, FilterType, PngEncoder}, load_from_memory, DynamicImage, ImageEncoder, ImageFormat};
use tokio::{io::AsyncWriteExt, runtime::Runtime, time::sleep};

// Types used by all command functions
type AsyncError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, AsyncError>;
type SContext = serenity::Context;
use log::{info, warn, debug, Level::Debug, set_max_level};
use failure::{Backtrace, Fail};
use std::str::FromStr;
use ttl_cache::TtlCache;

mod colors;

// Custom user data passed to all command functions


struct ImageCache {
    cache: Arc<Mutex<TtlCache<String, DynamicImage>>>,
}


impl ImageCache {
    fn new() -> Self {
        ImageCache {
            cache: Arc::new(Mutex::new(TtlCache::new(100))),
        }
    }

    async fn get(&self, url: &str) -> Option<DynamicImage> {
        println!("Checking cache for image");
        let cache = self.cache.lock().expect("cant access cache");
        println!("Cache: {}", cache.clone().iter().count());
        cache.get(&url.to_string()).cloned()
    }

    async fn insert(&self, url: String, image: DynamicImage) -> Option<()> {
        println!("Inserting image into cache");
        let mut cache = self.cache.lock().unwrap();
        println!("Cache insert before: {}", cache.clone().iter().count());
        cache.insert(url.clone(), image, Duration::from_secs(3600));
        println!("Cache insert after: {}", cache.clone().iter().count());
        
        Some(())
    }
}
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
    image_cache: Arc<ImageCache>,
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

async fn interaction_create(ctx: SContext, interaction: Interaction, data: &Data) -> Option<()> {
    if let Interaction::Component(interaction) = interaction {
        let content = &interaction.data.custom_id;
        if content.starts_with("darken-") {
            handle_interaction_darkening(&ctx, &interaction, data).await.unwrap();
        }
        if content.starts_with("delete-") {
            let message_id = content.split("-").last().unwrap().parse::<u64>().unwrap();
            handle_dispose(&ctx, &interaction, message_id).await.unwrap();
        }
        if content.starts_with("clear-") {
            initial_clear_components(&ctx, &interaction).await.unwrap();
        }
        if content.starts_with("stop-") {
            let response = CreateInteractionResponse::UpdateMessage(CreateInteractionResponseMessage::default());
            interaction.create_response(&ctx, response).await.unwrap();
            interaction.delete_response(&ctx).await.unwrap();
        }
    }
    Some(())
}

async fn handle_interaction_darkening(ctx: &SContext, interaction: &ComponentInteraction, data: &Data) -> Result<()> {
    let content = &interaction.data.custom_id;
    let options = parse_nord_custom_id(&content);
    let message_id = content.split("-").last().unwrap().parse::<u64>()?;
    let update = content.split("-").nth(1).unwrap().parse::<bool>().unwrap_or(true);
    let new_components = build_componets(message_id, options.clone(), true);
    // fetch message
    if let Err(err) = interaction.channel_id.message(&ctx, message_id).await {
        let response = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
            .content("Seems like the bright picture has vanished. I can't darken what I can't see.")
        );
        interaction.create_response(&ctx, response).await?;
    }
    let message = interaction.channel_id.message(&ctx, message_id).await?;
    if update {
        let response = CreateInteractionResponse::Acknowledge;
        interaction.create_response(&ctx, response).await?;
        let response = EditInteractionResponse::new()
            .attachments(EditAttachments::keep_all(&interaction.message))
            .content("⌛ I'm working on it. Please wait a moment.")
            .components(new_components.clone());
        interaction.edit_response(&ctx, response).await.unwrap();
    } else {
        //ack
        let response = CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
            .content("⌛ I'm working on it. Please wait a moment.")
            .components(new_components.clone())
        );
        interaction.create_response(&ctx, response).await.unwrap();
    }
    // sanduhr: 🕰️
    for attachment in &message.attachments {
        println!("Processing attachment");
        let image = process_image(&attachment, &message, ctx, data, options.clone()).await.unwrap();
        println!("writing image to buffer");
        let mut buffer = Vec::new();
            // Create a PNG encoder with a specific compression level
        {
            let mut cursor = Cursor::new(&mut buffer);
            let encoder = PngEncoder::new_with_quality(&mut cursor, CompressionType::Best, FilterType::NoFilter);
            encoder.write_image(&image.as_bytes(), image.width(), image.height(), image::ExtendedColorType::Rgba8).unwrap();
        }
        
        let attachment = CreateAttachment::bytes(buffer, "image.png");
        let content = EditInteractionResponse::new()
            .new_attachment(attachment)
            .content("Here it is! May I delete your shiny one?")
            .components(new_components.clone())
        ;
        // stone emoji: 
        println!("sending message");
        interaction.edit_response(&ctx, content).await?;
    }
    Ok(())
}


fn make_nord_custom_id(message_id: u64, update: bool, options: &colors::NordOptions) -> String {
    format!("darken-{}-{}-{}-{}-{}-{}", update, options.invert, options.hue_rotate, options.sepia, options.nord, message_id)
}

fn parse_nord_custom_id(custom_id: &str) -> colors::NordOptions {
    let mut parts = custom_id.split("-").skip(1);
    let _update = parts.next().unwrap().parse::<bool>().unwrap();
    let invert = parts.next().unwrap().parse::<bool>().unwrap();
    let hue_rotate = parts.next().unwrap().parse::<f32>().unwrap();
    let sepia = parts.next().unwrap().parse::<bool>().unwrap();
    let nord = parts.next().unwrap().parse::<bool>().unwrap();
    colors::NordOptions {invert, hue_rotate, sepia, nord}
}

fn build_componets(message_id: u64, options: colors::NordOptions, update: bool) -> Vec<CreateActionRow> {
    let mut components = Vec::new();
    let mut action_row = Vec::<CreateButton>::new();
    // make option lists, so that the clicked button is inverted
    let option_list = vec![
        ("Invert", options.invert, colors::NordOptions {invert: !options.invert, ..options}),
        ("Hue Rotate", if options.hue_rotate == 180. {true} else {false}, colors::NordOptions {hue_rotate: if options.hue_rotate == 180. {0.} else {180.}, ..options}),
        ("Sepia", options.sepia, colors::NordOptions {sepia: !options.sepia, ..options}),
        ("Nord", options.nord, colors::NordOptions {nord: !options.nord, ..options}),
    ];
    for (label, enabled, option) in option_list {
        action_row.push(
            CreateButton::new(make_nord_custom_id(message_id, update, &option))
                .style(ButtonStyle::Secondary)
                .label(&format!("{}", label))
                .style(if enabled {ButtonStyle::Primary} else {ButtonStyle::Secondary})
        );
    }
    components.push(CreateActionRow::Buttons(action_row));
    components.push(
        CreateActionRow::Buttons(
            vec![
                CreateButton::new(format!("delete-{}", message_id))
                    .style(ButtonStyle::Secondary)
                    .label("Dispose of the old!")
                    .emoji("🗑️".parse::<ReactionType>().unwrap()),
                // stop button
                CreateButton::new(format!("stop-{}", message_id))
                    .style(ButtonStyle::Secondary)
                    .label("Dispose of this"),
                CreateButton::new(format!("clear-{}", message_id))
                    .style(ButtonStyle::Secondary)
                    .label("Keep both")
            ]
        )
    );
    components
}

async fn initial_clear_components(ctx: &SContext, interaction: &ComponentInteraction) -> Result<()> {
    // fetch message
    let response = CreateInteractionResponse::Acknowledge;
    interaction.create_response(&ctx, response).await?;
    let response = EditInteractionResponse::new()
        .attachments(EditAttachments::keep_all(&interaction.message))
        .content("")
        .components(vec![]);
    interaction.edit_response(&ctx, response).await?;
    Ok(())
}

async fn handle_dispose(ctx: &SContext, interaction: &ComponentInteraction, message_id: u64) -> Result<()> {
    let content = &interaction.data.custom_id;
    initial_clear_components(&ctx, &interaction).await?;
    // fetch message
    interaction.channel_id.delete_message(&ctx, message_id).await?;
    let response =
        CreateInteractionResponseFollowup::new()
        .content("I have thrown it deep into the void to never see it again. Enjoy the darkness!")
        .ephemeral(true)
    ;
    interaction.create_followup(&ctx, response).await?;
    Ok(())
}


#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let image_cache = Arc::new(ImageCache::new());
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
                    image_cache: image_cache,
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
    ctx: &SContext,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, AsyncError>,
    data: &Data,
) -> Result<(), AsyncError> {
    println!(
        "Got an event in event handler: {:?}",
        event.snake_case_name()
    );

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::InteractionCreate { interaction, .. } => {
            interaction_create(ctx.clone(), interaction.clone(), data).await;
        }
        serenity::FullEvent::Message { new_message: message } => {
            for attachment in &message.attachments {
                if message.author.bot {
                    continue;
                }
                println!("attachment found");
                println!(
                    "media type: {:?}; filename: {}; Size: {} MiB; URL: {}", 
                    attachment.content_type, attachment.filename, attachment.size as f64 / 1024.0 / 1024.0, attachment.url
                );
                ask_user_to_darken_image(&ctx, &message, &attachment, data).await?;
            }
        }
        _ => {}
    }
    Ok(())
}


async fn image_check(attachment: &Attachment) -> Result<()> {
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
    Ok(())
}

async fn ask_user_to_darken_image(ctx: &SContext, message: &Message, attachment: &Attachment, data: &Data) -> Result<()> {
    image_check(attachment).await?;
    let url = attachment.url.clone();
    let image = {
        let image = data.image_cache.get(&url).await;
        if image.is_none() {
            download_image(&attachment).await
        } else {
            Ok(image.unwrap())
        }
    };
    println!("inserting");
    data.image_cache.insert(url, image.as_ref().unwrap().clone()).await;
    let bright = colors::calculate_average_brightness(&image?.to_rgba8());
    if bright < 0.4 {
        bail!("Not bright enough: {bright}")
    }
    let response = CreateMessage::new()
        .content(format!("Bruhh...\n\nThis looks bright as fuck. On a scale from 1 to 9 it's a {:.1}.\nMay I darken it?", bright*9.))
        .button(CreateButton::new(make_nord_custom_id(message.id.into(), false, &NordOptions::default()))
            .style(ButtonStyle::Primary)
            .emoji("🌙".parse::<ReactionType>().unwrap())
        )
        .button(CreateButton::new(format!("stop-{}", message.id))
            .style(ButtonStyle::Primary)
            .label("No")
        );
    let new_message = message.channel_id.send_message(ctx, response).await?;
    
    // Spawn a new task to delete the message after 5 minutes
    let ctx_clone = ctx.clone();
    let channel_id = message.channel_id;
    let message_id = message.id;

    // tokio::spawn(async move {
    //     sleep(Duration::from_secs(300)).await;
    //     if let Err(err) = new_message.delete(ctx_clone).await {
    //         eprintln!("Failed to delete message: {:?}", err);
    //     }
    // });

    Ok(())
}

async fn process_image(attachment: &serenity::Attachment, msg: &Message, ctx: &SContext, data: &Data, options: colors::NordOptions) -> Result<DynamicImage> {
    image_check(attachment).await?;
    let url = attachment.url.clone();
    let mut image = {
        let image = data.image_cache.get(&url).await;
        if image.is_none() {
            download_image(&attachment).await
        } else {
            Ok(image.unwrap())
        }
    };
    Ok(colors::apply_nord(image?, options))
}

async fn download_image(attachment: &Attachment) -> Result<DynamicImage> {
    // Send the GET request
    //println!("Downloading: {}=&format=png", attachment.proxy_url);
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