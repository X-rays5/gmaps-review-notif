using System.ComponentModel.DataAnnotations;
using Microsoft.EntityFrameworkCore;

namespace DataBase.Entity;

[PrimaryKey(nameof(Id))]
public class PostedReview
{
    public ulong Id { get; set; }

    [Required]
    [MinLength(1)]
    [MaxLength(100)]
    public required string PlaceId { get; set; }

    [Required]
    [MinLength(1)]
    [MaxLength(100)]
    public required string PlaceName { get; set; }

    [Required]
    public DateTime TimeCrawled { get; set; } = DateTime.UtcNow;

    [Required]
    public required int Stars { get;  set; }

    [Required]
    [MinLength(1)]
    [MaxLength(4200)]
    public required string ReviewBody { get;  set; }

    [Required]
    [MinLength(1)]
    [MaxLength(4200)]
    public required string ReviewBodyOriginal { get;  set; }

    [Required]
    [MinLength(1)]
    [MaxLength(100)]
    public required string GmapsUserId { get; set;  }
}
