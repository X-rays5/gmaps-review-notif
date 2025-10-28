namespace db.entity;

public class PostedReview
{
    public required string Id { get; set; }
    public DateTime TimeCrawled { get; set; } = DateTime.Now;
    public required Int32 stars { get;  set; }
    public required string reviewBody { get;  set; }
}
