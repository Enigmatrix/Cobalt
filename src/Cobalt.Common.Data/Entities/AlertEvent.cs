namespace Cobalt.Common.Data.Entities;

/// <summary>
///     An instance of <see cref="Alert" /> triggering.
/// </summary>
public class AlertEvent : IEntity
{
    public required Alert Alert { get; set; }
    public required DateTime Timestamp { get; set; }
    public long Id { get; set; }
}