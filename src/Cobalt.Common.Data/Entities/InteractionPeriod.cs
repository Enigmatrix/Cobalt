namespace Cobalt.Common.Data.Entities;

/// <summary>
///     A period of continuous usage without idling
/// </summary>
public class InteractionPeriod : IEntity, IHasDuration
{
    public required DateTime Start { get; set; }
    public required DateTime End { get; set; }
    public long MouseClicks { get; set; }
    public long KeyStrokes { get; set; }
    public long Id { get; set; }
    public TimeSpan Duration => End - Start;
}