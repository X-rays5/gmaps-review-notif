using Core;
using Discord;
using Discord.WebSocket;

namespace DiscordBot.Discord.Commands;

public class LatestReview : SlashCommandHandler
{
    public static SlashCommandProperties RegisterCommand()
    {
        var latestReviewCommand = new SlashCommandBuilder();
        latestReviewCommand.WithName("latest");
        latestReviewCommand.WithDescription("Get the latest review posted by a user");
        latestReviewCommand.WithDefaultMemberPermissions(GuildPermission.SendMessages);
        latestReviewCommand.AddOption("id", ApplicationCommandOptionType.String, "The gmaps user id of the user to get the latest review off", isRequired: true);
        latestReviewCommand.AddOption("original", ApplicationCommandOptionType.Boolean, "Whether to get the review in its original language", isRequired: false);

        return latestReviewCommand.Build();
    }

    public static async Task HandleCommand(SocketSlashCommand command)
    {
        string gmapsUserId = null!;
        bool getOriginal = true;

        await command.Data.Options.ToAsyncEnumerable().ForEachAsync(option =>
        {
            switch (option.Name)
            {
                case "id":
                    gmapsUserId = (string)option.Value;
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

        var postedReview = await GmapsUserService.GetGmapsUserLastPostedReview(await GmapsUserService.GetGmapsUserById(gmapsUserId));
        if (postedReview == null)
        {
            await command.FollowupAsync("No reviews found for this user.");
            return;
        }

        await command.FollowupAsync(embed: Utilities.PostedReviewToEmbed(postedReview, await GmapsUserService.GetGmapsUserById(gmapsUserId), getOriginal).Build());
    }
}
