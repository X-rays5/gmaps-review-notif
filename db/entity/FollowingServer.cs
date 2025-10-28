using System.ComponentModel.DataAnnotations;

namespace db.entity;

public class FollowingServer
{
    public int Id { get; set; } // auto pk
    public required ulong GuildId { get; set; }
    public required ulong ChannelId { get; set; }

    public required GmapsUser GmapsUser { get; set; }
}
