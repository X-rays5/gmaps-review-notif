using Core;
using Discord.WebSocket;
using DiscordBot.Discord;
using log4net;

namespace DiscordBot;

public static class Init
{
    private static readonly ILog Logger = LogManager.GetLogger(typeof(Init));

    public static async Task Connect()
    {
        var client = new Client();
        await client.Connect();

        var backgroundThread = new Thread(BackgroundWorker);
        backgroundThread.IsBackground = true;
        backgroundThread.Start();
    }

    private static async void BackgroundWorker()
    {
        while (true)
        {
            Logger.Info("Starting periodic update checker...");
            var updaterThread = await PeriodicUpdateCheckerService.StartUpdatedWorkerAsync();

            while (updaterThread.IsAlive)
            {
                Logger.Debug("Waiting for updater thread to complete...");
                await Task.Delay(TimeSpan.FromMinutes(1));
            }

            Logger.Info("Updater thread completed. Sending reviews from queue...");
            await SentReviewsFromQueue();

            updaterThread.Join();
            Logger.Info("Periodic update checker completed. Sleeping for 10 minutes...");
            Thread.Sleep(TimeSpan.FromMinutes(10));
        }
    }

    private static async Task SentReviewsFromQueue()
    {
        while (PeriodicUpdateCheckerService.NewReviewsQueue.TryDequeue(out var reviewDto))
        {
            var followingServers = await FollowingServersService.GetServersFollowingUser(reviewDto.GmapsUserId);
            foreach (var server in followingServers)
            {
                Logger.Debug("Sending review " + reviewDto.Id + " to server " + server.GuildId);

                SocketGuild? guild = Client.DiscordClient.GetGuild(server.GuildId);
                if (guild == null)
                {
                    await FollowingServersService.StopFollowingUserInServer(server.GuildId, reviewDto.GmapsUserId);
                    continue;
                }

                SocketTextChannel? channel = guild?.GetTextChannel(server.ChannelId);
                if (channel == null)
                {
                    await FollowingServersService.StopFollowingUserInServer(server.GuildId, reviewDto.GmapsUserId);
                    continue;
                }

                await channel.SendMessageAsync(embed: Utilities.PostedReviewToEmbed(reviewDto, server.GetOriginal).Build());
            }
        }
    }
}
