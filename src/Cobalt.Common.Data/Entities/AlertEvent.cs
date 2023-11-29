namespace Cobalt.Common.Data.Entities;

public class AlertEvent
{
    public long Id { get; set; }
    public required Alert Alert { get; set; }
    public required DateTime Timestamp { get; set; }
}