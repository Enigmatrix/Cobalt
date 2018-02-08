using System;
using System.ComponentModel;
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
    }
}