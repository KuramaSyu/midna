use poise::{CreateReply, ReplyHandle};
use serenity::all::{CreateAttachment, Message, User};

use crate::{colors::NordOptions, fetch_image_and_info, process_attachments, AsyncError, Context};

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), AsyncError> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(context_menu_command = "Edit Image", slash_command)]
pub async fn edit_message_image(
    ctx: Context<'_>,
    #[description = "test"] message: Message,
) -> Result<(), AsyncError>{
    let reply = ctx.reply("Building ...").await?;
    if message.attachments.len() == 0 {
        reply.edit(ctx, CreateReply::default().content("No image found")).await?;
        return Ok(());
    }
    reply.edit(ctx, CreateReply::default().content("Downloading image ...")).await?;
    let first_attachment = message.attachments.first().unwrap();
    let (_image, info) = fetch_image_and_info(&first_attachment, ctx.data()).await?;
    reply.edit(ctx, CreateReply::default().content("Processing image ...")).await?;
    let mut options = NordOptions::from_image_information(&info);
    options.start = true;
    let buffer = process_attachments(&message, ctx.data(), &options).await?;
    reply.edit(ctx, CreateReply::default().content("Uploading image ...")).await?;
    ctx.send(
        CreateReply::default()
            .attachment(CreateAttachment::bytes(buffer, &first_attachment.filename))
            .components(options.build_componets(u64::from(message.id), true))
    ).await?;
    reply.delete(ctx).await?;
    Ok(())
}

