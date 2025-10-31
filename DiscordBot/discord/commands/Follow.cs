using Core;
using Discord;
using Discord.WebSocket;
using DtoMappers;

namespace DiscordBot.Discord.Commands;

internal class Follow : SlashCommandHandler
{
    private enum FollowOption
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
        string gmapsUserId = null!;
        var followOption = FollowOption.None;

        await command.Data.Options.ToAsyncEnumerable().ForEachAsync(option =>
        {
            switch (option.Name)
            {
                case "id":
                    gmapsUserId = (string)option.Value;
                    break;
                case "enable":
                    followOption = (bool)option.Value ? FollowOption.Follow : FollowOption.Unfollow;
                    break;
            }
        });

        if (string.IsNullOrWhiteSpace(gmapsUserId))
        {
            command.RespondAsync("You must provide a valid Google Maps User ID").Wait();
            return;
        }

        await command.DeferAsync();

        var gmapsUser = await GmapsUserService.GetGmapsUserById(gmapsUserId);
        var isServerFollowing = await FollowingServersService.IsUserFollowedInServer(command.GuildId ?? 0, gmapsUserId);

        switch (followOption)
        {
            case FollowOption.None:
                await HandleNoneFollowOption(command, gmapsUserId, gmapsUser, isServerFollowing);
                break;
            case FollowOption.Follow:
                await HandleFollowServerOption(command, gmapsUserId, gmapsUser, isServerFollowing);
                break;
            case FollowOption.Unfollow:
                await HandleUnfollowServerOption(command, gmapsUserId, gmapsUser, isServerFollowing);
                break;
        }
    }

    private static async Task HandleNoneFollowOption(SocketSlashCommand command, string gmapsUserId, GmapsUserDto gmapsUser, bool isServerFollowing)
    {
        if (isServerFollowing)
        {
            await FollowingServersService.StopFollowingUserInServer(command.GuildId ?? 0, gmapsUserId);
            await command.FollowupAsync($"Stopped following user: {gmapsUser.Name}");
            return;
        }

        await FollowingServersService.StartFollowingUserInServer(command.GuildId ?? 0, command.ChannelId ?? 0, gmapsUserId);
        await command.FollowupAsync($"Now following user: {gmapsUser.Name}");
    }

    private static async Task HandleFollowServerOption(SocketSlashCommand command, string gmapsUserId, GmapsUserDto gmapsUser, bool isServerFollowing)
    {
        if (isServerFollowing)
        {
            await command.FollowupAsync($"This server is already following user: {gmapsUser.Name}");
            return;
        }

        await FollowingServersService.StartFollowingUserInServer(command.GuildId ?? 0, command.ChannelId ?? 0, gmapsUserId);
        await command.FollowupAsync($"Now following user: {gmapsUser.Name}");
    }

    private static async Task HandleUnfollowServerOption(SocketSlashCommand command, string gmapsUserId, GmapsUserDto gmapsUser, bool isServerFollowing)
    {
        if (!isServerFollowing)
        {
            await command.FollowupAsync($"This server is not following user: {gmapsUser.Name}");
            return;
        }

        await FollowingServersService.StopFollowingUserInServer(command.GuildId ?? 0, gmapsUserId);
        await command.FollowupAsync($"Stopped following user: {gmapsUser.Name}");
    }
}
