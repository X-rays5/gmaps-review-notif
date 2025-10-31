namespace DtoMappers;

public class GmapsUserDto
{
    public required string Id { get; set; }
    public required string Name { get; set; }
    public List<FollowingServerDto> FollowingServers { get; set; } = new();
    public PostedReviewDto? LatestPostedReview { get; set; }
}
