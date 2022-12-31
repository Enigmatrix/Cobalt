namespace Cobalt.Common.Utils;

public class ToDiscriminatedUnionException<T> : Exception
{
    public ToDiscriminatedUnionException(int tag) : base($"Discriminated Union ({typeof(T)}) with unknown tag {tag}") {}
}