namespace Cobalt.Common.Utils;

public class FromDiscriminatedUnionException<T> : Exception
{
    public FromDiscriminatedUnionException() : base($"Discriminated Union ({typeof(T)}) with unknown branch") {}
}