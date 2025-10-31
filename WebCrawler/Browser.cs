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
                { "Accept-Language", "en-US,en;q=0.9,nl;q=0.8" }
            }
        });
        Page = await _context.NewPageAsync();
    }

    public async ValueTask DisposeAsync()
    {
        if (_context != null) await _context.CloseAsync();
        if (_browser != null) await _browser.CloseAsync();
        _playwright?.Dispose();
    }

    public async Task AcceptGoogleTerms()
    {
        var termsAccepts = Page.GetByRole(AriaRole.Button, new PageGetByRoleOptions{ Name = "Accept all", Exact = false });
        if (await termsAccepts.IsVisibleAsync())
            await termsAccepts.ClickAsync();

        termsAccepts = Page.GetByRole(AriaRole.Button, new PageGetByRoleOptions{ Name = "Alles accepteren", Exact = false });
        if (await termsAccepts.IsVisibleAsync())
            await termsAccepts.ClickAsync();
    }
}
