using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace DataBase.Entity;

[PrimaryKey(nameof(Id))]
public class GmapsUser
{
    [Required]
    [MinLength(1)]
    [MaxLength(100)]
    public required string Id { get; set; }

    [Required]
    [MinLength(1)]
    [MaxLength(100)]
    public required string Name { get; set; }

    public List<FollowingServer> FollowingServers { get; set; } = new();

    public ulong? LatestPostedReviewId { get; set; }
    [ForeignKey(nameof(LatestPostedReviewId))]
    public PostedReview? LatestPostedReview { get; set; }
}
