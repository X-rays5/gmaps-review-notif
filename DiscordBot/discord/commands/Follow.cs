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
        var followUserCommand = new SlashCommandBuilder();
        followUserCommand.WithName("follow");
        followUserCommand.WithDescription("Start following a new user in this channel");
        followUserCommand.WithDefaultMemberPermissions(GuildPermission.ManageWebhooks);
        followUserCommand.AddOption("id", ApplicationCommandOptionType.String, "The gmaps user id of the user to follow", isRequired: true);
        followUserCommand.AddOption("enable", ApplicationCommandOptionType.Boolean, "Enable or disable notifications for this user in this channel", isRequired: false);
        followUserCommand.AddOption("original", ApplicationCommandOptionType.Boolean, "Whether to get the review in its original language", isRequired: false);

        return followUserCommand.Build();
    }

    public static async Task HandleCommand(SocketSlashCommand command)
    {
        string gmapsUserId = null!;
        var followOption = FollowOption.None;
        bool getOriginal = false;

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
                case "original":
                    getOriginal = (bool)option.Value;
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
                await HandleNoneFollowOption(command, gmapsUserId, gmapsUser, getOriginal, isServerFollowing);
                break;
            case FollowOption.Follow:
                await HandleFollowServerOption(command, gmapsUserId, gmapsUser, getOriginal, isServerFollowing);
                break;
            case FollowOption.Unfollow:
                await HandleUnfollowServerOption(command, gmapsUserId, gmapsUser, isServerFollowing);
                break;
        }
    }

    private static async Task HandleNoneFollowOption(SocketSlashCommand command, string gmapsUserId, GmapsUserDto gmapsUser, bool getOriginal, bool isServerFollowing)
    {
        if (isServerFollowing)
        {
            await FollowingServersService.StopFollowingUserInServer(command.GuildId ?? 0, gmapsUserId);
            await command.FollowupAsync($"Stopped following user: {gmapsUser.Name}");
            return;
        }

        await FollowingServersService.StartFollowingUserInServer(new FollowingServerDto
        {
            GuildId = command.GuildId ?? 0,
            ChannelId = command.ChannelId ?? 0,
            GmapsUserId = gmapsUserId,
            GetOriginal = getOriginal
        });
        await command.FollowupAsync($"Now following user: {gmapsUser.Name}");
    }

    private static async Task HandleFollowServerOption(SocketSlashCommand command, string gmapsUserId, GmapsUserDto gmapsUser, bool getOrignal, bool isServerFollowing)
    {
        if (isServerFollowing)
        {
            await command.FollowupAsync($"This server is already following user: {gmapsUser.Name}");
            return;
        }

        await FollowingServersService.StartFollowingUserInServer(new FollowingServerDto
        {
            GuildId = command.GuildId ?? 0,
            ChannelId = command.ChannelId ?? 0,
            GmapsUserId = gmapsUserId,
            GetOriginal = getOrignal
        });
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
