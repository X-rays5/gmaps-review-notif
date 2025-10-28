using System.ComponentModel.DataAnnotations;

namespace db.entity;

public class GmapsUser
{
    [MaxLength(100)]
    public required string Id { get; set; }
    [MaxLength(100)]
    public required string Name { get; set; }
    public required PostedReview LatestPostedReview { get; set; }
    public List<FollowingServer> FollowingServers { get; set; } = new();
}
