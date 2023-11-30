namespace Cobalt.Common.Data.Entities;

public class AlertEvent : IEntity
{
    public required Alert Alert { get; set; }
    public required DateTime Timestamp { get; set; }
    public long Id { get; set; }
}