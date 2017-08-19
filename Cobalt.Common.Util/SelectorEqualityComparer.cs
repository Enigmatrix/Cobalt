using System;
using System.Collections.Generic;

namespace Cobalt.Common.Util
{
    public class SelectorEqualityComparer<T, TR> : IEqualityComparer<T>
    {
        public SelectorEqualityComparer(Func<T, TR> selector)
        {
            Selector = selector;
        }

        public Func<T, TR> Selector { get; set; }

        public bool Equals(T x, T y)
        {
            return Selector(x).Equals(Selector(y));
        }

        public int GetHashCode(T obj)
        {
            return Selector(obj).GetHashCode();
        }
    }
}