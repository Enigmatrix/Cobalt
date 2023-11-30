using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Tests;

public class DataTests : IDisposable
{
    private static int _dbCount;
    private readonly QueryContext _context;

    public DataTests()
    {
        var count = Interlocked.Increment(ref _dbCount);
        var optionsBuilder = new DbContextOptionsBuilder<QueryContext>();
        QueryContext.ConfigureFor(optionsBuilder, $"Data Source=test-{count}.db");
        _context = new QueryContext(optionsBuilder.Options);
        _context.Database.EnsureCreated();
    }

    public void Dispose()
    {
        var connection = _context.Database;
        _context.Dispose();
        connection.EnsureDeleted();
    }

    [Fact]
    public void Setup_Succeeds()
    {
        Assert.True(true);
    }
}