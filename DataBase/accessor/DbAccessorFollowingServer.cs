using DataBase.Entity;
using Microsoft.EntityFrameworkCore;

namespace DataBase.Accessor;

public class DbAccessorFollowingServer : DbDisposable<AppDbContext>
{
    public DbAccessorFollowingServer(AppDbContext? dbContext = null) : base(dbContext ?? new AppDbContext())
    {}

    public async Task<List<FollowingServer>> GetServersFollowingGmapsUserAsync(string gmapsUserId)
    {
        return await _context.FollowingServers
            .Where(fs => fs.GmapsUserId == gmapsUserId)
            .ToListAsync();
    }

    public async Task<bool> IsUserFollowedInServer(ulong guildId, string gmapsUserId)
    {
        return await _context.FollowingServers
            .Where(fs => fs.GuildId == guildId && fs.GmapsUserId == gmapsUserId)
            .AnyAsync();
    }

    public void AddFollowingServer(FollowingServer followingServer)
    {
        _context.FollowingServers.Add(followingServer);
    }

    public void RemoveFollowingServer(FollowingServer followingServer)
    {
        _context.FollowingServers.Remove(followingServer);
    }
}
