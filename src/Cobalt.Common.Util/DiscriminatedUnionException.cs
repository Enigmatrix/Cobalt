namespace Cobalt.Common.Util;

public class DiscriminatedUnionException<T> : Exception
{
    public DiscriminatedUnionException(string name, T? value = default) : base(
        $"Invalid Discriminated Union state {typeof(T)} for {name}: {value}")
    {
        Value = value;
    }

    public T? Value { get; }
}