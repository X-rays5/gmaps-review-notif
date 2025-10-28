using crawler;
using db.entity;
using Discord;
using Discord.WebSocket;

namespace discord_bot.discord.commands;

public class Follow : SlashCommandHandler
{
    enum FollowOption
    {
        None,
        Follow,
        Unfollow
    }

    public static SlashCommandProperties RegisterCommand()
    {
        var followSwitch = new SlashCommandBuilder();
        followSwitch.WithName("follow");
        followSwitch.WithDescription("Start following a new user in this channel");
        followSwitch.WithDefaultMemberPermissions(GuildPermission.ManageWebhooks);
        followSwitch.AddOption("id", ApplicationCommandOptionType.String, "The gmaps user id of the user to follow", isRequired: true);
        followSwitch.AddOption("enable", ApplicationCommandOptionType.Boolean, "Enable or disable notifications for this user in this channel", isRequired: false);

        return followSwitch.Build();
    }

    public static async Task HandleCommand(SocketSlashCommand command)
    {
        string gmapsUserID = null;
        FollowOption followOption = FollowOption.None;

        await command.Data.Options.ToAsyncEnumerable().ForEachAsync(option =>
        {
            switch (option.Name)
            {
                case "id":
                    gmapsUserID = (string)option.Value;
                    break;
                case "enable":
                    followOption = ((bool)option.Value) ? FollowOption.Follow : FollowOption.Unfollow;
                    break;
            }
        });

        if (string.IsNullOrWhiteSpace(gmapsUserID ?? ""))
        {
            command.RespondAsync("You must provide a valid Google Maps User ID").Wait();
            return;
        }

        await command.DeferAsync();

        await using var dbCtx = new db.AppDbContext();
        var gmapsUser = await GetUser.Execute(dbCtx, gmapsUserID!);

        var server = gmapsUser.FollowingServers.Find(fs => fs.GuildId == command.GuildId);
        if (server != null)
        {
            if (followOption == FollowOption.Follow)
            {
                await command.FollowupAsync("You are already following this user in this server.");
                return;
            }

            gmapsUser.FollowingServers.Remove(server);
            await dbCtx.SaveChangesAsync();
            await command.FollowupAsync($"Unfollowed user: {gmapsUser.Name}");
            return;
        }

        if (followOption == FollowOption.Unfollow)
        {
            await command.FollowupAsync("You are not following this user in this server.");
            return;
        }

        gmapsUser.FollowingServers.Add(new FollowingServer
        {
            GuildId = command.GuildId ?? 0,
            ChannelId = command.ChannelId ?? 0
        });
        await dbCtx.SaveChangesAsync();

        await command.FollowupAsync($"Now following user: {gmapsUser.Name}");
    }
}
