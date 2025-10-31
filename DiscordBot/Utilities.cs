using System.Text;
using Discord;
using DtoMappers;

namespace DiscordBot;

public static class Utilities
{
    public static EmbedBuilder PostedReviewToEmbed(PostedReviewDto reviewDto, bool getOriginal = false)
    {
        var starsField = new EmbedFieldBuilder();
        StringBuilder starsBuilder = new StringBuilder();
        for (int i = 0; i < reviewDto.Stars; i++)
        {
            starsBuilder.Append("<:starrating:1433928079805517874>");
        }
        starsField.Name = "Stars";
        starsField.Value = starsBuilder.ToString();

        var bodyField = new EmbedFieldBuilder();
        bodyField.Name = "Review";
        bodyField.Value = getOriginal ? reviewDto.ReviewBodyOriginal : reviewDto.ReviewBody;

        var footer = new EmbedFooterBuilder();
        footer.Text = "There may be a delay of up to 3 hours for the latest review to be fetched.";

        var builder = new EmbedBuilder();
        builder.Title = "Latest Review";
        builder.Fields.Add(starsField);
        builder.Fields.Add(bodyField);
        builder.Footer = footer;

        return builder;
    }
}
