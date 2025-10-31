using DataBase.Accessor;
using DataBase.Entity;

namespace Core;

public static class FollowingServersService
{
    public static async Task<bool> IsUserFollowedInServer(ulong guildId, string gmapsUserId)
    {
        if (guildId < 1 || string.IsNullOrWhiteSpace(gmapsUserId))
        {
            return false;
        }

        await using var dbContext = new DbAccessorFollowingServer();
        return await dbContext.IsUserFollowedInServer(guildId, gmapsUserId);
    }

    public static async Task StopFollowingUserInServer(ulong guildId, string gmapsUserId)
    {
        if (guildId < 1 || string.IsNullOrWhiteSpace(gmapsUserId))
        {
            return;
        }

        await using var dbContext = new DbAccessorFollowingServer();
        var servers = await dbContext.GetServersFollowingGmapsUserAsync(gmapsUserId);
        var server = servers.FirstOrDefault(s => s.GuildId == guildId);
        if (server != null)
        {
            dbContext.RemoveFollowingServer(server);
            await dbContext.SaveChangesAsync();
        }
    }

    public static async Task StartFollowingUserInServer(ulong guildId, ulong channelId, string gmapsUserId)
    {
        if (guildId < 1 || channelId < 1 || string.IsNullOrWhiteSpace(gmapsUserId))
        {
            return;
        }

        var user = await GmapsUserService.GetGmapsUserById(gmapsUserId);

        await using var dbContext = new DbAccessorFollowingServer();
        var followingServer = new FollowingServer
        {
            GuildId = guildId,
            ChannelId = channelId,
            GmapsUserId = user.Id
        };
        dbContext.AddFollowingServer(followingServer);
        await dbContext.SaveChangesAsync();
    }
}
