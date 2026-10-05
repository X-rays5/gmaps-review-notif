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
            on_error: |error| {
                Box::pin(async move {
                    if let poise::FrameworkError::Command { error, ctx, .. } = &error {
                        // Invocation (command + args) is already logged at INFO by `log_interaction`;
                        // here we only record that it failed and where.
                        tracing::error!(
                            command = %ctx.command().qualified_name,
                            discord_user = %ctx.author().name,
                            discord_user_id = ctx.author().id.get(),
                            guild_id = ?ctx.guild_id().map(|id| id.get()),
                            channel_id = ctx.channel_id().get(),
                            error = %error,
                            "Command failed"
                        );
                    }

                    if let Err(e) = poise::builtins::on_error(error).await {
                        tracing::error!(error = %e, "Failed to report command error to the user");
                    }
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
    let serenity::FullEvent::InteractionCreate { interaction } = event else {
        return;
    };
    if interaction.kind() != InteractionType::Command {
        return;
    }
    let Some(command) = interaction.as_command() else {
        return;
    };

    tracing::info!(
        command = %command.data.name,
        args = %format_command_args(&command.data.options),
        discord_user = %command.user.name,
        discord_user_id = command.user.id.get(),
        guild_id = ?command.guild_id.map(|id| id.get()),
        channel_id = ?command.channel.as_ref().map(|channel| channel.id.get()),
        "Slash command received"
    );
}

/// Renders command options as a compact `name=value name=[subcommand…]` string for logging.
fn format_command_args(options: &[serenity::CommandDataOption]) -> String {
    options
        .iter()
        .map(|option| match &option.value {
            serenity::CommandDataOptionValue::SubCommand(inner)
            | serenity::CommandDataOptionValue::SubCommandGroup(inner) => {
                format!("{}=[{}]", option.name, format_command_args(inner))
            }
            value => format!("{}={}", option.name, format_command_arg(value)),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_command_arg(value: &serenity::CommandDataOptionValue) -> String {
    match value {
        serenity::CommandDataOptionValue::Autocomplete { value, .. } => value.clone(),
        serenity::CommandDataOptionValue::Boolean(value) => value.to_string(),
        serenity::CommandDataOptionValue::Integer(value) => value.to_string(),
        serenity::CommandDataOptionValue::Number(value) => value.to_string(),
        serenity::CommandDataOptionValue::String(value) => value.clone(),
        serenity::CommandDataOptionValue::Attachment(id) => id.get().to_string(),
        serenity::CommandDataOptionValue::Channel(id) => id.get().to_string(),
        serenity::CommandDataOptionValue::Mentionable(id) => id.get().to_string(),
        serenity::CommandDataOptionValue::Role(id) => id.get().to_string(),
        serenity::CommandDataOptionValue::User(id) => id.get().to_string(),
        serenity::CommandDataOptionValue::Unknown(kind) => format!("unknown({kind})"),
        // Subcommands are handled by `format_command_args`; this is a safety net for
        // future/unknown option kinds.
        other => format!("{other:?}"),
    }
}
