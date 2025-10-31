using DataBase.Accessor;
using DataBase.Entity;
using DtoMappers;
using DtoMappers.Mappers;
using WebCrawler;

namespace Core;

internal static class PostedReviewService
{
    public static async Task<PostedReviewDto?> UpdateLatestReviewForUser(GmapsUserDto gmapsUser)
    {
        await using var dbAccessor = new DbAccessorPostedReview();
        var dbContext = dbAccessor.GetDbContext();

        await using var transaction = await dbContext.Database.BeginTransactionAsync();

        var latestPostedReviewInDb = await dbAccessor.GetLatestPostedReviewForUser(
            GmapsUserMapper.GmapsUserDtoToEntity(gmapsUser));

        if (latestPostedReviewInDb != null && DateTime.UtcNow - latestPostedReviewInDb.TimeCrawled < TimeSpan.FromHours(3))
            return null;

        var latestReview = await UserLatestReview.Execute(gmapsUser);
        if (latestReview == null)
            return null;

        if (latestPostedReviewInDb != null)
            dbAccessor.RemovePostedReview(latestPostedReviewInDb);

        var latestReviewEntity = PostedReviewMapper.PostedReviewDtoToEntity(latestReview);
        dbAccessor.AddPostedReview(latestReviewEntity);

        // Fetch the user and update the navigation property directly
        var userEntity = await dbContext.Set<GmapsUser>().FindAsync(gmapsUser.Id);
        if (userEntity != null)
        {
            userEntity.LatestPostedReview = latestReviewEntity;
        }

        await dbContext.SaveChangesAsync();

        await transaction.CommitAsync();

        return PostedReviewMapper.PostedReviewToDto(latestReviewEntity);
    }
}
