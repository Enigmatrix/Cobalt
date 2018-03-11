using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using Xunit;

namespace Cobalt.Tests
{
    public class AssertEx
    {
        public static void DeepEquals<T>(T self, T to, params string[] ignore) where T : class
        {
            DeepEquals(self, to, typeof(T), ignore);
        }

        public static void DeepEquals(object self, object to, Type type, params string[] ignore)
        {
            if (self != null && to != null)
            {
                var ignoreList = new List<string>(ignore);
                var allProps = type.GetProperties(BindingFlags.Public | BindingFlags.Instance);
                var props = (
                    from pi in allProps
                    where !ignoreList.Contains(pi.Name)
                    let selfValue = type.GetProperty(pi.Name).GetValue(self, null)
                    let toValue = type.GetProperty(pi.Name).GetValue(to, null)
                    where !(selfValue == toValue ||
                            selfValue != null && selfValue.Equals(toValue) ||
                            selfValue.GetType().GetProperties(BindingFlags.Public | BindingFlags.Instance).Count() !=
                            0 && selfValue.GetType() != typeof(object) && selfValue.GetType() == toValue.GetType() &&
                            IsDeepEquals(selfValue, toValue, selfValue.GetType()))
                    select (selfValue, toValue, pi.Name)).ToList();
                if (props.Count == 0) return;
                throw new AggregateException(props.Select(
                    x => new Exception(
                        $"Property {x.Item3} does not match in object {self}, left: {x.Item1} | right: {x.Item2}")));
            }

            Assert.Equal(self, to);
        }

        public static bool IsDeepEquals(object self, object to, Type type)
        {
            DeepEquals(self, to, type);
            return true;
        }
    }
}