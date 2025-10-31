using Microsoft.Playwright;

namespace WebCrawler;

public class Browser : IAsyncDisposable
{
    private IPlaywright _playwright;
    private IBrowser _browser;
    private IBrowserContext _context;
    public IPage Page { get; private set; }

    public async Task InitAsync()
    {
        _playwright = await Playwright.CreateAsync();
        _browser = await _playwright.Chromium.LaunchAsync(new BrowserTypeLaunchOptions
        {
            Headless = false,
            Args = new[] { "--no-sandbox" }
        });
        _context = await _browser.NewContextAsync(new BrowserNewContextOptions()
        {
            Locale = "en-US", // still sets navigator.language
            ExtraHTTPHeaders = new Dictionary<string, string>
            {
                { "Accept-Language", "en-US,en;q=1.0" }
            }
        });
        Page = await _context.NewPageAsync();

        await HandleGoogleTerms();
    }

    public async ValueTask DisposeAsync()
    {
        if (_context != null) await _context.CloseAsync();
        if (_browser != null) await _browser.CloseAsync();
        _playwright?.Dispose();
    }

    private async Task HandleGoogleTerms()
    {
        await Page.GotoAsync("https://google.com/maps?hl=en");

        await Page.WaitForURLAsync(url => url.Contains("consent.google.com"));

        while (Page.Url.Contains("consent.google.com"))
        {
            var termsAccepts = Page.GetByRole(AriaRole.Button, new PageGetByRoleOptions{ Name = "Accept all", Exact = false });
            await termsAccepts.WaitForAsync();
            if (await termsAccepts.IsVisibleAsync())
                await termsAccepts.ClickAsync();

            await Page.WaitForTimeoutAsync(1000);
        }
    }
}
