using System.Text.RegularExpressions;
using db;
using db.entity;
using Microsoft.EntityFrameworkCore;

namespace crawler;

public class GetUser
{
    private static readonly string GmapsUserUrl = "https://www.google.com/maps/contrib/{0}/reviews";

    public static async Task<GmapsUser> Execute(AppDbContext dbCtx, string userId)
    {
        var gmapsUser = await dbCtx.GmapsUsers
            .Include(u => u.FollowingServers)
            .FirstOrDefaultAsync(u => u.Id == userId);

        if (gmapsUser == null)
        {
            gmapsUser = await FetchFromGmaps(userId);
            dbCtx.GmapsUsers.Add(gmapsUser);
            gmapsUser.FollowingServers = new List<FollowingServer>();
            await dbCtx.SaveChangesAsync();
        }

        return gmapsUser;
    }

    private static async Task<GmapsUser> FetchFromGmaps(string userId)
    {
        string url = string.Format(GmapsUserUrl, userId);

        await using var browser = new Browser();
        await browser.InitAsync();

        await browser.Page.GotoAsync(url);
        await browser.AcceptGoogleTerms();

        await browser.Page.WaitForURLAsync(new Regex(@"\/reviews\/@.*"));

        var nameHeader = browser.Page.Locator("h1.fontHeadlineLarge[role='button'][tabindex='0'][aria-haspopup='true']");
        string userName = await nameHeader.InnerTextAsync();

        return new GmapsUser
        {
            Id = userId,
            Name = userName,
            LatestPostedReview = await UserLatestReview.GetLatestReview(userId)
        };
    }
}
