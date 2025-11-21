use poise::serenity_prelude as serenity;
use anyhow::{Result, Error};
use crate::discord::commands;

pub async fn build(token: String) -> Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::follow_user(), commands::lookup_user(), commands::latest_review()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                ctx.set_presence(Some(serenity::ActivityData::custom("Watching for Google Maps Reviews")), serenity::OnlineStatus::DoNotDisturb);

                Ok(())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework).await;

    match client {
        Ok(c) => Ok(c),
        Err(e) => Err(Error::msg(format!("Failed to create Discord client: {}", e))),
    }
}