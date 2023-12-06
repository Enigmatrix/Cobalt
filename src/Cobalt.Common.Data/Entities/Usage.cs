namespace Cobalt.Common.Data.Entities;

/// <summary>
///     A continuous usage of an <see cref="App" /> during an <see cref="Session" />
/// </summary>
public class Usage : IEntity, IHasDuration
{
    public required Session Session { get; set; }
    public required DateTime Start { get; set; }
    public required DateTime End { get; set; }

    public long Id { get; set; }

    public TimeSpan Duration => End - Start;
}