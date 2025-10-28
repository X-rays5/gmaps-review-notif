using db;
using discord_bot;
using log4net;
using log4net.Config;
using Microsoft.EntityFrameworkCore;

namespace gmaps_review_notif;

internal static class Program
{
    private static readonly ILog LOG = LogManager.GetLogger(typeof(Program));

    private static async Task Main(string[] args)
    {
        DotEnv.Load(".env");
        SetupLogger();

        await using var db = new AppDbContext();
        await db.Database.EnsureCreatedAsync();
        await db.Database.MigrateAsync();
        
        await Init.Connect();
        
        await Task.Delay(-1);
    }

    private static void SetupLogger()
    {
        BasicConfigurator.Configure();
    }
}
