using crawler;
using db.entity;
using Discord;
using Discord.WebSocket;

namespace discord_bot.discord.commands;

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
        String gmapsUserId = null;

        await command.Data.Options.ToAsyncEnumerable().ForEachAsync(async option =>
        {
            switch (option.Name)
            {
                case "id":
                    gmapsUserId = (String)option.Value;
                    break;
            }
        });

        if (String.IsNullOrWhiteSpace(gmapsUserId ?? ""))
        {
            command.RespondAsync("You must provide a valid Google Maps User ID").Wait();
            return;
        }

        await command.DeferAsync();

        PostedReview postedReview = await UserLatestReview.GetLatestReview(gmapsUserId);
        EmbedFieldBuilder starsField = new EmbedFieldBuilder();
        starsField.Name = "Stars";
        starsField.Value = new string('⭐', postedReview.stars);

        EmbedFieldBuilder bodyField = new EmbedFieldBuilder();
        bodyField.Name = "Review";
        bodyField.Value = postedReview.reviewBody;

        EmbedBuilder builder = new EmbedBuilder();
        builder.Title = "Latest Review";
        builder.Fields.Add(starsField);
        builder.Fields.Add(bodyField);

        await command.FollowupAsync(embed: builder.Build());
    }
}
