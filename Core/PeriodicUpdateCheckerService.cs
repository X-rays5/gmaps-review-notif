using System.Collections.Concurrent;
using DataBase.Entity;
using DtoMappers;
using DtoMappers.Mappers;
using log4net;

namespace Core;

public static class PeriodicUpdateCheckerService
{
    private static readonly ILog Logger = LogManager.GetLogger(typeof(PeriodicUpdateCheckerService));

    public static ConcurrentQueue<PostedReviewDto> NewReviewsQueue { get; } = new();

    public static async Task<Thread> StartUpdatedWorkerAsync()
    {
        List<GmapsUser> followedUsers;
        await using (var dbAccessorGmapsUser = new DataBase.Accessor.DbAccessorGmapsUser())
        {
            followedUsers = await dbAccessorGmapsUser.GetUsersWithFollowersAsync();
        }

        // We start a new thread for each user which will slowly go through all users
        Thread workerThread = new(async void () =>
        {
            try
            {
                foreach (var gmapsUser in followedUsers)
                {
                    try
                    {
                        Logger.Info($"Checking for updates for user {gmapsUser.Id}...");
                        var gmapsUserDto = GmapsUserMapper.GmapsUserToDto(gmapsUser);

                        if (gmapsUserDto.LatestPostedReview != null && DateTime.UtcNow - gmapsUserDto.LatestPostedReview.TimeCrawled < TimeSpan.FromHours(6))
                        {
                            Logger.Info($"Skipping user {gmapsUser.Id} as their latest review was crawled recently.");
                            continue;
                        }

                        var latestPostedReview = await PostedReviewService.UpdateLatestReviewForUser(gmapsUserDto);
                        if (latestPostedReview != null)
                        {
                            Logger.Info($"New review found for user {gmapsUser.Id}: {latestPostedReview.Id}");

                            NewReviewsQueue.Enqueue(latestPostedReview);
                        }
                        else
                        {
                            Logger.Info($"No new review for user {gmapsUser.Id}.");
                        }
                    }
                    catch (Exception ex)
                    {
                        Logger.Error($"Error while checking updates for user {gmapsUser.Id}: ", ex);
                    }

                    Logger.Debug("Sleeping for 10 seconds before checking the next user...");
                    await Task.Delay(TimeSpan.FromSeconds(10));
                }
            }
            catch (Exception e)
            {
                Logger.Error("Error in PeriodicUpdateCheckerService worker thread: ", e);
            }
        });

        workerThread.IsBackground = true;
        workerThread.Start();
        return workerThread;
    }
}
