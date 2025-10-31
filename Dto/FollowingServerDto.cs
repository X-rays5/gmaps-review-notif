namespace DtoMappers;

public class FollowingServerDto
{
    public ulong Id { get; set; }
    public required ulong GuildId { get; set; }
    public required ulong ChannelId { get; set; }
    public required bool GetOriginal { get; set; }
    public required string GmapsUserId { get; set; }
}
