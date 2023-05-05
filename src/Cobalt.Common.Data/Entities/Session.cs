namespace Cobalt.Common.Data.Entities;

public class Session
{
    public long Id { get; set; }
    public App App { get; set; }
    public string Title { get; set; }
    public string? CmdLine { get; set; }
}
