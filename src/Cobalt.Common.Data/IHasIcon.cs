namespace Cobalt.Common.Data;

public interface IHasIcon : IDisposable
{
    public Stream Icon { get; }
}