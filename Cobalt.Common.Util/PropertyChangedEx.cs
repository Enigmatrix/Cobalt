using System;
using System.ComponentModel;
using System.Reactive;
using System.Reactive.Linq;

namespace Cobalt.Common.Util
{
    public static class PropertyChangedEx
    {
        public static IObservable<string> PropertyChanges(this INotifyPropertyChanged npc)
        {
            return Observable.FromEventPattern<PropertyChangedEventHandler, PropertyChangedEventArgs>(
                    handler => handler.Invoke, h => npc.PropertyChanged += h, h => npc.PropertyChanged -= h)
                .Select(x => x.EventArgs.PropertyName);
        }
        public static IObservable<Unit> PropertyChanges(this INotifyPropertyChanged npc, string prop)
        {
            var changes =  Observable.FromEventPattern<PropertyChangedEventHandler, PropertyChangedEventArgs>(
                    handler => handler.Invoke, h => npc.PropertyChanged += h, h => npc.PropertyChanged -= h)
                .Select(x => x.EventArgs.PropertyName)
                .Where(x => x == prop)
                .Select(x => Unit.Default);
            return Observable.Return(Unit.Default).Concat(changes);
        }
    }
}