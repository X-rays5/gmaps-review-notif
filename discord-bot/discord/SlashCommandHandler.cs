using Discord;
using Discord.WebSocket;

namespace discord_bot.discord;

public interface SlashCommandHandler
{
    public static abstract SlashCommandProperties RegisterCommand();
    public static abstract Task HandleCommand(SocketSlashCommand command);
}
