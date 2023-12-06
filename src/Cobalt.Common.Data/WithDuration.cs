namespace Cobalt.Common.Data;

/// <summary>
///     Wrapper around an <see cref="IEntity" /> to add a <paramref name="Duration" /> field
/// </summary>
/// <typeparam name="T">Wrapped type</typeparam>
/// <param name="Inner">Wrapped value</param>
/// <param name="Duration">Duration value</param>
public record struct WithDuration<T>(T Inner, TimeSpan Duration) : IHasDuration, IHasInner<T>;