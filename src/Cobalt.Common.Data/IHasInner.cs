namespace Cobalt.Common.Data;

/// <summary>
///     Trait interface for wrapper classes
/// </summary>
public interface IHasInner<out T>
{
    public T Inner { get; }
}