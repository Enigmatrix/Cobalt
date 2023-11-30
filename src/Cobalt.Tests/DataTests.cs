using Cobalt.Common.Data;

namespace Cobalt.Tests;

public class DataTests : IDisposable
{
    private static int _dbCount;
    private readonly QueryContext _context;

    public DataTests()
    {
        var count = Interlocked.Increment(ref _dbCount);
        var path = $"test-{count}.db";
        _context = new QueryContext(path);
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