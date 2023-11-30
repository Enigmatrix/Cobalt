namespace Cobalt.Common.Data.Entities;

public class InteractionPeriod : IEntity
{
    public required DateTime Start { get; set; }
    public required DateTime End { get; set; }
    public long MouseClicks { get; set; }
    public long KeyStrokes { get; set; }
    public long Id { get; set; }
}