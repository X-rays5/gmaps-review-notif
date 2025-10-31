using DataBase.Accessor;
using DtoMappers;
using DtoMappers.Mappers;
using WebCrawler;

namespace Core;

internal static class PostedReviewService
{
    public static async Task<PostedReviewDto?> UpdateLatestReviewForUser(GmapsUserDto gmapsUser)
    {
        await using var dbAccessorPostedReview = new DbAccessorPostedReview();
        var dbContext = dbAccessorPostedReview.GetDbContext();

        await using var transaction = await dbContext.Database.BeginTransactionAsync();

        var latestPostedReviewInDb = await dbAccessorPostedReview.GetLatestPostedReviewForUser(
            GmapsUserMapper.GmapsUserDtoToEntity(gmapsUser));

        if (latestPostedReviewInDb != null && DateTime.UtcNow - latestPostedReviewInDb.TimeCrawled < TimeSpan.FromHours(3))
            return null;

        var latestReview = await UserLatestReview.Execute(gmapsUser);
        if (latestReview == null)
            return null;

        if (latestPostedReviewInDb != null)
            dbAccessorPostedReview.RemovePostedReview(latestPostedReviewInDb);

        var latestReviewEntity = PostedReviewMapper.PostedReviewDtoToEntity(latestReview);
        latestReviewEntity = dbAccessorPostedReview.AddPostedReview(latestReviewEntity);

        await using var dbAccessorGmapsUser = new DbAccessorGmapsUser(dbContext);
        await dbAccessorGmapsUser.UpdateLatestPostedReviewAsync(gmapsUser.Id, latestReviewEntity);

        await dbAccessorPostedReview.SaveChangesAsync();
        await dbAccessorGmapsUser.SaveChangesAsync();
        await transaction.CommitAsync();

        return PostedReviewMapper.PostedReviewToDto(latestReviewEntity);
    }

}
