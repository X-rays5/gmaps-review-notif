using db.entity;
using Microsoft.EntityFrameworkCore;

namespace db;

public class AppDbContext : DbContext
{
    public DbSet<GmapsUser> GmapsUsers { get; set; } = null!;
    public DbSet<FollowingServer> FollowingServers { get; set; } = null!;

    protected override void OnConfiguring(DbContextOptionsBuilder options)
    {
        var dbProvider = Environment.GetEnvironmentVariable("DB_PROVIDER") ?? "SQLite";

        if (dbProvider == "SQLite")
        {
            var sqlitePath = Environment.GetEnvironmentVariable("SQLITE_PATH") ?? "app.db";
            options.UseSqlite($"Data Source={sqlitePath}");
        }
        else
        {
            var conn = Environment.GetEnvironmentVariable("ConnectionStrings__Default") ?? "Host=db;Database=exampledb;Username=postgres;Password=password";
            options.UseNpgsql(conn);
        }
    }
}
