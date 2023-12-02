namespace Cobalt.Common.Data;

public record struct WithDuration<T>(T Inner, TimeSpan Duration) : IHasDuration, IHasInner<T>;