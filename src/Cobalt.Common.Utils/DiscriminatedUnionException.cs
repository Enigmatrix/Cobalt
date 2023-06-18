namespace Cobalt.Common.Utils;

public class DiscriminatedUnionException<T> : Exception
{
    public DiscriminatedUnionException(string name) : base($"Invalid Discriminated Union state for {typeof(T)}: {name}")
    {
    }
}