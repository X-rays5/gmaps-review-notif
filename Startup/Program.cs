using DataBase;
using DiscordBot;
using log4net;
using log4net.Config;
using Microsoft.EntityFrameworkCore;

namespace Startup;

internal static class Program
{
    private static readonly ILog Logger = LogManager.GetLogger(typeof(Program));

    private static async Task Main(string[] args)
    {
        DotEnv.Load(".env");
        SetupLogger();

        Logger.Info("[INIT] Database migration starting...");
        await using var db = new AppDbContext();
        await db.Database.EnsureCreatedAsync();
        await db.Database.MigrateAsync();

        Logger.Info("[INIT] Connecting to discord");
        await Init.Connect();

        Logger.Info("[INIT] Initialization complete, entering wait state");
        await Task.Delay(-1);
    }

    private static void SetupLogger()
    {
        BasicConfigurator.Configure();
    }
}
