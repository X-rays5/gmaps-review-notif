using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace DataBase.Entity;

[PrimaryKey(nameof(Id))]
public class FollowingServer
{
    public int Id { get; set; } // auto pk
    [Required]
    public required ulong GuildId { get; set; }
    [Required]
    public required ulong ChannelId { get; set; }

    [Required]
    [MinLength(1)]
    [MaxLength(100)]
    public required string GmapsUserId { get; set; }
}
