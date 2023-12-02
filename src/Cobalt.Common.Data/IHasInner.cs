namespace Cobalt.Common.Data;

public interface IHasInner<out T>
{
    public T Inner { get; }
}