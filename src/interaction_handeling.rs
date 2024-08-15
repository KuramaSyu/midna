use serenity::all::{ComponentInteraction, CreateAttachment, CreateInteractionResponse, CreateInteractionResponseFollowup, EditAttachments, EditInteractionResponse, Message, ModalInteraction};
use anyhow::Result;
use crate::{colors::{NordOptions, RgbColor}, fetch_image, fetch_or_raise_message, modal_get_color, process_attachments, AnyInteraction, Data, SContext};


/// Handles an interaction starting with dark-
/// which will modify the image
pub async fn handle_interaction_darkening(
    ctx: &SContext, 
    interaction: &ComponentInteraction, 
    data: &Data
) -> Result<()> {
    data.question_messages.lock().await.remove(&interaction.message.id.into());
    let content = &interaction.data.custom_id;
    let mut options = NordOptions::from_custom_id(&content);
    let message_id = content.split("-").last().unwrap().parse::<u64>()?;
    let _update = content.split("-").nth(1).unwrap().parse::<bool>().unwrap_or(true);

    let mut current_interaction = AnyInteraction::Component(interaction.clone());

    // ask for background color
    if options.background_color == Some(RgbColor::from_hex("000001").unwrap()) {
        let color: RgbColor;
        let new_interaction: ModalInteraction;
        (color, new_interaction) = match modal_get_color(&ctx, &interaction).await {
            Ok(color) => color,
            Err(_) => {
                // Error handled inside modal_get_color
                return Ok(());
            }
        };
        options.background_color = Some(color);
        current_interaction = AnyInteraction::Modal(new_interaction);
    }
    let mut message: Option<Message> = None;

    // auto adjust options
    if options.auto_adjust {
        message = Some(fetch_or_raise_message(&ctx, &interaction, message_id).await);
        let ref unwrapped = message.as_ref().unwrap();
        let (_image, information) = fetch_image(unwrapped.attachments.first().unwrap(), data).await;
        let new_options = NordOptions::from_image_information(&information);
        options = NordOptions {start: options.start, ..new_options};
    }

    let new_components = options.build_componets(message_id, true);
    
    println!("options: {:?}", options);

    if options.start {
        // start button pressed
        if message.is_none() {
            message = Some(fetch_or_raise_message(&ctx, &interaction, message_id).await);
        }
        let response = CreateInteractionResponse::Acknowledge;
        current_interaction.create_response(&ctx, response).await?;
        // edit response with new components
        let response = EditInteractionResponse::new()
            .attachments(EditAttachments::keep_all(&interaction.message))
            .content("⌛ I'm working on it. Please wait a moment.")
            .components(new_components.clone());
        current_interaction.edit_response(&ctx, response).await.unwrap();
    } else {
        // first ack, that existing image is being kept
        let response = CreateInteractionResponse::Acknowledge;
        current_interaction.create_response(&ctx, response).await?;
        // edit response with new components
        let response = EditInteractionResponse::new()
            .attachments(EditAttachments::keep_all(&interaction.message))
            .content("⌛ I change the options. Please wait a moment.")
            .components(new_components.clone());
        current_interaction.edit_response(&ctx, response).await.unwrap();
    }
    
    if !options.start {
        let response = EditInteractionResponse::new()
            .content("Edited your options.")
            .components(new_components.clone())
        ;
        current_interaction.edit_response(&ctx, response).await?;
        return Ok(())
    }
    // ensure existence of message
    if message.is_none() {
        current_interaction.edit_response(&ctx, EditInteractionResponse::new()
            .content("Seems like the bright picture has vanished. I can't darken what I can't see.")
        ).await?;
        return Ok(())
    }
    let message = message.unwrap();
    // process image
    let buffer = match process_attachments(&message, &data, &options).await {
        Ok(buffer) => buffer,
        Err(e) => {
            current_interaction.edit_response(&ctx, EditInteractionResponse::default().content(e.to_string())).await?;
            return Ok(())
        }
    };
    let attachment = CreateAttachment::bytes(buffer, "image.webp");
    let content = EditInteractionResponse::new()
        .new_attachment(attachment)
        .content("Here it is! May I delete your shiny one?")
        .components(new_components.clone())
    ;
    // stone emoji: 
    println!("sending message");
    current_interaction.edit_response(&ctx, content).await?;
    Ok(())
}

/// handeles interactions starting with delete-
/// which will delete the message_id which is contained in the custom_id
pub async fn handle_dispose(ctx: &SContext, interaction: &ComponentInteraction, message_id: u64) -> Result<()> {
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

/// clears the components of the given interaction.
pub async fn initial_clear_components(ctx: &SContext, interaction: &ComponentInteraction) -> Result<()> {
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