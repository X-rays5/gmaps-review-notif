using DataBase.Accessor;
using DtoMappers;
using DtoMappers.Mappers;

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

    public static async Task StartFollowingUserInServer(FollowingServerDto followingServerDto)
    {
        if (followingServerDto.GuildId < 1 || followingServerDto.ChannelId < 1 || string.IsNullOrWhiteSpace(followingServerDto.GmapsUserId))
        {
            return;
        }

        // Ensure the GmapsUser exists
        await GmapsUserService.GetGmapsUserById(followingServerDto.GmapsUserId);

        await using var dbContext = new DbAccessorFollowingServer();
        dbContext.AddFollowingServer(FollowingServerMapper.FollowingServerToEntity(followingServerDto));
        await dbContext.SaveChangesAsync();
    }

    public static async Task<List<FollowingServerDto>> GetServersFollowingUser(string gmapsUserId)
    {
        if (string.IsNullOrWhiteSpace(gmapsUserId))
        {
            return new List<FollowingServerDto>();
        }

        await using var dbContext = new DbAccessorFollowingServer();
        var servers = await dbContext.GetServersFollowingGmapsUserAsync(gmapsUserId);
        return servers.Select(FollowingServerMapper.FollowingServerToDto).ToList();
    }
}
