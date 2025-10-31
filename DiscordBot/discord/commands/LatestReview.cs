using Core;
using Discord;
using Discord.WebSocket;

namespace DiscordBot.Discord.Commands;

public class LatestReview : SlashCommandHandler
{
    public static SlashCommandProperties RegisterCommand()
    {
        var galnetFeedSwitch = new SlashCommandBuilder();
        galnetFeedSwitch.WithName("latest");
        galnetFeedSwitch.WithDescription("Get the latest review posted by a user");
        galnetFeedSwitch.WithDefaultMemberPermissions(GuildPermission.SendMessages);
        galnetFeedSwitch.AddOption("id", ApplicationCommandOptionType.String, "The gmaps user id of the user to get the latest review off", isRequired: true);

        return galnetFeedSwitch.Build();
    }

    public static async Task HandleCommand(SocketSlashCommand command)
    {
        string gmapsUserId = null!;

        await command.Data.Options.ToAsyncEnumerable().ForEachAsync(option =>
        {
            switch (option.Name)
            {
                case "id":
                    gmapsUserId = (string)option.Value;
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

        var starsField = new EmbedFieldBuilder();
        starsField.Name = "Stars";
        starsField.Value = new string('⭐', postedReview.Stars);

        var bodyField = new EmbedFieldBuilder();
        bodyField.Name = "Review";
        bodyField.Value = postedReview.ReviewBody;

        var footer = new EmbedFooterBuilder();
        footer.Text = "There may be a delay of up to 3 hours for the latest review to be fetched.";

        var builder = new EmbedBuilder();
        builder.Title = "Latest Review";
        builder.Fields.Add(starsField);
        builder.Fields.Add(bodyField);
        builder.Footer = footer;

        await command.FollowupAsync(embed: builder.Build());
    }
}
