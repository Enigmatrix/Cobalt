using System.Runtime.CompilerServices;

namespace Cobalt.Common.Util
{
    public class Cache<TKey, TValue>
        where TKey : class
        where TValue : class
    {
        private readonly ConditionalWeakTable<TKey, TValue> _internalCache;

        public Cache()
        {
            _internalCache = new ConditionalWeakTable<TKey, TValue>();
        }

        public void Add(TKey key, TValue val)
        {
            _internalCache.Add(key, val);
        }

        public bool TryGetValue(TKey key, out TValue val)
        {
            return _internalCache.TryGetValue(key, out val);
        }
    }
}