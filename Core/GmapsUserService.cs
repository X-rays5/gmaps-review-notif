using DataBase.Accessor;
using DataBase.Entity;
using DtoMappers;
using DtoMappers.Mappers;
using log4net;
using WebCrawler;

namespace Core;

public static class GmapsUserService
{
    private static readonly ILog Logger = LogManager.GetLogger(typeof(GmapsUserService));

    public static async Task<GmapsUserDto> GetGmapsUserById(string gmapsUserId)
    {
        if (string.IsNullOrWhiteSpace(gmapsUserId))
        {
            throw new ArgumentException("Gmaps user ID cannot be null or whitespace.", nameof(gmapsUserId));
        }

        await using var dbContext = new DbAccessorGmapsUser();
        GmapsUserDto user;
        GmapsUser? userDb = await dbContext.GetGmapsUserByIdAsync(gmapsUserId);
        if (userDb == null)
        {
            var newUser = await GetUser.Execute(gmapsUserId);
            dbContext.AddGmapsUser(GmapsUserMapper.GmapsUserDtoToEntity(newUser));
            await dbContext.SaveChangesAsync();
            user = newUser;
        }
        else
        {
            user = GmapsUserMapper.GmapsUserToDto(userDb);
        }

        var latestPostedReview = await GetGmapsUserLastPostedReview(user);
        if (latestPostedReview != null)
        {
            user.LatestPostedReviewId = latestPostedReview.Id;
        }

        return user;
    }

    public static async Task<PostedReviewDto?> GetGmapsUserLastPostedReview(GmapsUserDto user)
    {
        var postedReview = await PostedReviewService.UpdateLatestReviewForUser(user);
        if (postedReview != null)
        {
            return postedReview;
        }

        await using var dbAccessorPostedReview = new DbAccessorPostedReview();
        var postedReviewEntity = await dbAccessorPostedReview.GetLatestPostedReviewForUser(GmapsUserMapper.GmapsUserDtoToEntity(user));
        if (postedReviewEntity == null)
        {
            return null;
        }

        return PostedReviewMapper.PostedReviewToDto(postedReviewEntity);
    }
}
