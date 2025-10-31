using DataBase.Entity;
using Microsoft.EntityFrameworkCore;

namespace DataBase.Accessor;

public class DbAccessorPostedReview : DbDisposable<AppDbContext>
{
    public DbAccessorPostedReview(AppDbContext? dbContext = null) : base(dbContext ?? new AppDbContext())
    {}

    public async Task<PostedReview?> GetLatestPostedReviewForUser(GmapsUser gmapsUser)
    {
        return await _context.PostedReviews
            .Where(r => r.GmapsUserId == gmapsUser.Id)
            .OrderByDescending(r => r.TimeCrawled)
            .FirstOrDefaultAsync();
    }

    public PostedReview AddPostedReview(PostedReview postedReview)
    {
        return _context.PostedReviews.Add(postedReview).Entity;
    }

    public PostedReview RemovePostedReview(PostedReview postedReview)
    {
        return _context.PostedReviews.Remove(postedReview).Entity;
    }
}
