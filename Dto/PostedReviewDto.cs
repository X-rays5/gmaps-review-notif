namespace DtoMappers;

public class PostedReviewDto
{
    public ulong Id { get; set; }
    public required string PlaceId { get; set; }
    public DateTime TimeCrawled { get; set; } = DateTime.UtcNow;
    public required int Stars { get;  set; }
    public required string ReviewBody { get;  set; }
    public GmapsUserDto GmapsUser { get; set; }
}
