using DataBase.Entity;
using Microsoft.EntityFrameworkCore;

namespace DataBase;

public class AppDbContext : DbContext
{
    public DbSet<GmapsUser> GmapsUsers { get; set; } = null!;
    public DbSet<FollowingServer> FollowingServers { get; set; } = null!;
    public DbSet<PostedReview> PostedReviews { get; set; } = null!;

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
    {
        optionsBuilder.EnableSensitiveDataLogging();
        optionsBuilder.EnableDetailedErrors();

        var dbProvider = Environment.GetEnvironmentVariable("DB_PROVIDER") ?? "SQLite";

        if (dbProvider == "SQLite")
        {
            var sqlitePath = Environment.GetEnvironmentVariable("SQLITE_PATH") ?? "app.db";
            optionsBuilder.UseSqlite($"Data Source={sqlitePath}");
        }
        else
        {
            var conn = Environment.GetEnvironmentVariable("ConnectionStrings__Default") ?? "Host=DataBase;Database=exampledb;Username=postgres;Password=password";
            optionsBuilder.UseNpgsql(conn);
        }
    }
}
