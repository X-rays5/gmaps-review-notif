using Discord;
using Discord.WebSocket;

namespace DiscordBot.Discord.Commands;

public class Help : SlashCommandHandler
{
    public static SlashCommandProperties RegisterCommand()
    {
        var helpCommand = new SlashCommandBuilder();
        helpCommand.WithName("help");
        helpCommand.WithDescription("Instructions on how to use the bot");
        helpCommand.WithDefaultMemberPermissions(GuildPermission.SendMessages);

        return helpCommand.Build();
    }

    public static async Task HandleCommand(SocketSlashCommand command)
    {
        await command.DeferAsync();

        var helpMessage = "To use this bot you will need someone's Google Maps User ID. You can find this by going to their Google Maps profile and copying the string of characters in the URL after 'https://www.google.com/maps/contrib/'.";

        await command.FollowupAsync(helpMessage);
    }
}
