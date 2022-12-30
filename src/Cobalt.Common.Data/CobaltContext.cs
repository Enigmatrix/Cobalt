using Cobalt.Common.Data.Models;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.Data;

public class CobaltContext : DbContext
{
    private readonly string _path;

    /// <summary>
    /// Initialize CobaltContext with a connection string file path
    /// </summary>
    /// <param name="path">file path</param>
    public CobaltContext(string path)
    {
        _path = path;
    }

    public DbSet<App> Apps { get; init; } = default!;

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
    {
        optionsBuilder.UseSqlite($"Data Source={_path}");
    }
}
