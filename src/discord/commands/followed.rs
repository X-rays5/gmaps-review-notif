use crate::discord::commands::{ack, CommandCtx};
use crate::models::User;
use crate::provider;
use anyhow::Result;

/// Gets a list of users being followed in the current channel.
#[poise::command(
    slash_command,
    rename = "followed",
    default_member_permissions = "MANAGE_WEBHOOKS",
    required_bot_permissions = "MANAGE_WEBHOOKS"
)]
pub async fn followed_command<U: Sync>(ctx: CommandCtx<'_, U>) -> Result<()> {
    ack(&ctx).await;

    let users = match provider::following::get_users_followed_in_channel(
        ctx.channel_id().get().to_string(),
    ) {
        Ok(u) => u,
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to retrieve followed users: {}", e))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let Some(content) = format_followed_users(&users) else {
        ctx.send(
            poise::CreateReply::default()
                .content("ℹ️ No users are currently being followed in this channel.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    ctx.send(
        poise::CreateReply::default()
            .content(content)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

fn format_followed_users(users: &[User]) -> Option<String> {
    if users.is_empty() {
        return None;
    }

    let mut content = String::from("📋 **Followed Users in this Channel:**\n");
    for user in users {
        content.push_str(&format!("- {} (`{}`)\n", user.name, user.gmaps_id));
    }
    Some(content)
}

#[cfg(test)]
mod tests {
    use super::format_followed_users;
    use crate::models::User;

    #[test]
    fn format_followed_users_returns_none_for_empty_list() {
        assert!(format_followed_users(&[]).is_none());
    }

    #[test]
    fn format_followed_users_formats_single_user() {
        let users = vec![User {
            id: 1,
            gmaps_id: "abc123".to_string(),
            name: "Alice".to_string(),
        }];

        assert_eq!(
            format_followed_users(&users).as_deref(),
            Some("📋 **Followed Users in this Channel:**\n- Alice (`abc123`)\n")
        );
    }

    #[test]
    fn format_followed_users_formats_multiple_users() {
        let users = vec![
            User {
                id: 1,
                gmaps_id: "abc123".to_string(),
                name: "Alice".to_string(),
            },
            User {
                id: 2,
                gmaps_id: "def456".to_string(),
                name: "Bob".to_string(),
            },
        ];

        assert_eq!(
            format_followed_users(&users).as_deref(),
            Some(
                "📋 **Followed Users in this Channel:**\n- Alice (`abc123`)\n- Bob (`def456`)\n"
            )
        );
    }
}

