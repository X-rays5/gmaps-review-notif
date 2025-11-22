use crate::discord::commands::*;
use anyhow::{Error, Result};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::InteractionType;

pub async fn build(token: String) -> Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                follow::follow_user(),
                followed::followed_command(),
                latest::latest_review(),
                lookup::lookup_user(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(async move {
                    log_interaction(ctx, event, framework, data);
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                ctx.set_presence(
                    Some(serenity::ActivityData::custom(
                        "Watching for Google Maps Reviews",
                    )),
                    serenity::OnlineStatus::DoNotDisturb,
                );

                Ok(())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(c) => Ok(c),
        Err(e) => Err(Error::msg(format!(
            "Failed to create Discord client: {}",
            e
        ))),
    }
}

fn log_interaction(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, (), Error>,
    _data: &(),
) {
    if let serenity::FullEvent::InteractionCreate { interaction } = event {
        if interaction.kind() == InteractionType::Command {
            let interaction = interaction.as_command().unwrap();
            tracing::info!(
                "Slash command: user='{}({})', guild='{}', channel='{}', command='{}'",
                interaction.user.name,
                interaction.user.id,
                interaction.guild_id.unwrap().get(),
                interaction.channel.clone().unwrap().id.get(),
                interaction.data.name
            );
        }
    }
}
