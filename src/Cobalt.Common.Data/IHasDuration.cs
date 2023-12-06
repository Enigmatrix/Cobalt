namespace Cobalt.Common.Data;

/// <summary>
///     Trait interface for classes with a <see cref="Duration" />
/// </summary>
public interface IHasDuration
{
    public TimeSpan Duration { get; }
}