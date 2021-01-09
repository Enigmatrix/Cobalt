using System;
using System.Collections.Generic;

namespace Cobalt.Common.Utils
{
    public class WeakValueCache<TKey, TValue>
        where TValue : class
        where TKey : notnull
    {
        private readonly Dictionary<TKey, WeakReference<TValue>> _inner;

        public WeakValueCache()
        {
            _inner = new Dictionary<TKey, WeakReference<TValue>>();
        }

        public TValue? this[TKey index] => Get(index);

        public TValue? Get(TKey key)
        {
            if (!_inner.TryGetValue(key, out var r)) return null;

            if (r.TryGetTarget(out var v)) return v;

            _inner.Remove(key);
            return null;
        }

        public void Set(TKey key, TValue val)
        {
            _inner[key] = new WeakReference<TValue>(val);
        }
    }
}