using DataBase.Entity;
using Microsoft.EntityFrameworkCore;

namespace DataBase.Accessor;

public class DbAccessorGmapsUser : DbDisposable<AppDbContext>
{
    public DbAccessorGmapsUser(AppDbContext? dbContext = null) : base(dbContext ?? new AppDbContext())
    {}

    public async Task<GmapsUser?> GetGmapsUserByIdAsync(string gmapsUserId)
    {
        return await _context.GmapsUsers
            .Include(u => u.FollowingServers)
            .FirstOrDefaultAsync(u => u.Id == gmapsUserId);
    }

    public void AddGmapsUser(GmapsUser gmapsUser)
    {
        _context.GmapsUsers.Add(gmapsUser);
    }

    public async Task UpdateLatestPostedReviewAsync(string gmapsUserId, PostedReview postedReview)
    {
        var gmapsUser = await _context.GmapsUsers
            .Where(u => u.Id == gmapsUserId)
            .FirstOrDefaultAsync();

        if (gmapsUser != null)
        {
            gmapsUser.LatestPostedReviewId = postedReview.Id;
        }
    }

    public async Task<List<GmapsUser>> GetUsersWithFollowersAsync()
    {
        return await _context.GmapsUsers
            .Where(u => u.FollowingServers.Any())
            .Include(u => u.FollowingServers)
            .ToListAsync();
    }
}
