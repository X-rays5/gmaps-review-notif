using System.Text.RegularExpressions;
using DtoMappers;

namespace WebCrawler;

public class GetUser
{
    private static readonly string GmapsUserUrl = "https://www.google.com/maps/contrib/{0}/reviews?hl=en";

    public static async Task<GmapsUserDto> Execute(string userId)
    {
        string url = string.Format(GmapsUserUrl, userId);

        await using var browser = new Browser();
        await browser.InitAsync();

        await browser.Page.GotoAsync(url);

        await browser.Page.WaitForURLAsync(new Regex(@"\/reviews\/@.*"));

        var nameHeader = browser.Page.Locator("h1.fontHeadlineLarge[role='button'][tabindex='0'][aria-haspopup='true']");
        string userName = await nameHeader.InnerTextAsync();

        return new GmapsUserDto
        {
            Id = userId,
            Name = userName,
        };
    }
}
