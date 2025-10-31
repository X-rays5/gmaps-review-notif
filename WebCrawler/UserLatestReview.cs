using System.Text.RegularExpressions;
using DtoMappers;
using log4net;

namespace WebCrawler;

public class UserLatestReview
{
    private static readonly ILog LOG = LogManager.GetLogger(typeof(UserLatestReview));

    private static string placeIDRegex = @"\/place\/([a-zA-Z0-9-_]+)";

    public static async Task<PostedReviewDto?> Execute(GmapsUserDto user)
    {
        await using var browser = new Browser();
        await browser.InitAsync();

        await browser.Page.GotoAsync($"https://www.google.com/maps/contrib/{user.Id}/reviews?hl=en");

        await browser.Page.WaitForURLAsync(new Regex(@"\/reviews\/@.*"));

        await browser.Page.ClickAsync("div.jftiEf:nth-child(1)");
        await browser.Page.WaitForURLAsync(new Regex(@"\/place\/[a-zA-Z0-9-_]+/@.*"));

        LOG.Info($"Current URL: {browser.Page.Url}");
        var m = Regex.Match(browser.Page.Url, placeIDRegex);
        string? placeID = null;
        if (m.Success)
        {
            placeID = m.Groups[1].Value;
        }

        var reviewBodySpan = browser.Page.Locator("div[tabindex='-1'] > span");
        if (await reviewBodySpan.CountAsync() == 0)
        {
            LOG.Warn("No review body found");
            return null;
        }

        string reviewBody = await reviewBodySpan.First.InnerTextAsync();
        LOG.Info($"Review body: {reviewBody}");

        string reviewBodyOriginal = reviewBody;
        var seeOriginalSpan = browser.Page.Locator("span:has-text('See original')").First;
        if (await seeOriginalSpan.CountAsync() > 0)
        {
            await seeOriginalSpan.ClickAsync();
            // re-fetch the review body
            reviewBodySpan = browser.Page.Locator("div[tabindex='-1'] > span");
            reviewBodyOriginal = await reviewBodySpan.First.InnerTextAsync();
            LOG.Info($"Review body (original): {reviewBody}");
        }

        var starSpan = browser.Page.Locator("span[role='img'][aria-label]:has(span.google-symbols)");
        int starRating = 0;
        if (await starSpan.CountAsync() > 0)
        {
            // now we get the aria-label and take everything before the first space as that is the number of stars
            string ariaLabel = await starSpan.First.GetAttributeAsync("aria-label");
            LOG.Info($"Star aria-label: {ariaLabel}");
            var starMatch = Regex.Match(ariaLabel, @"^([0-9]+)");
            if (starMatch.Success)
            {
                starRating = int.Parse(starMatch.Groups[1].Value);
            }
        }

        return new PostedReviewDto
        {
            PlaceId = placeID,
            ReviewBody = reviewBody,
            ReviewBodyOriginal = reviewBodyOriginal,
            Stars = starRating,
            GmapsUser = user
        };
    }
}
