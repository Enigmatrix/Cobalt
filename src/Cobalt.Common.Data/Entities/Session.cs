namespace Cobalt.Common.Data.Entities;

public class Session
{
    public long Id { get; set; }
    public required string Title { get; set; }
    public required App App { get; set; }
}