using Discord;
using Discord.WebSocket;

namespace DiscordBot.Discord;

public interface SlashCommandHandler
{
    public static abstract SlashCommandProperties RegisterCommand();
    public static abstract Task HandleCommand(SocketSlashCommand command);
}
