using System;
using System.Collections.Generic;
using System.Collections.Specialized;
using System.Linq;
using System.Reactive.Linq;
using Avalonia.Collections;
using DynamicData;

namespace Cobalt.Extensions;

/// <summary>
///     Extension class for <see cref="INotifyCollectionChanged" />
/// </summary>
public static class NotifyCollectionChangedEx
{
    /// <summary>
    ///     Similar to <see cref="NotifyCollectionChangedExtensions.GetWeakCollectionChangedObservable" />
    ///     combined with
    ///     <see cref="DynamicData.Binding.ObservableCollectionEx.ToObservableChangeSet{TCollection, T}(TCollection)" />
    /// </summary>
    /// <typeparam name="TCollection">The type of collection.</typeparam>
    /// <typeparam name="T">The type of the object.</typeparam>
    /// <param name="source">The source.</param>
    /// <returns>An observable that emits the change set.</returns>
    public static IObservable<IChangeSet<T>> GetWeakCollectionChangeSet<TCollection, T>(
        this TCollection source) where TCollection : INotifyCollectionChanged, IEnumerable<T>
        where T : notnull
    {
        var weakCollectionChangedObservable = source.GetWeakCollectionChangedObservable();

        // ref: https://github.com/reactivemarbles/DynamicData/blob/2217b591027424be0c160ca4e5c7f18419f42c63/src/DynamicData/Binding/ObservableCollectionEx.cs#L117
        return Observable.Create<IChangeSet<T>>(
            observer =>
            {
                var data = new ChangeAwareList<T>(source);

                if (data.Count > 0) observer.OnNext(data.CaptureChanges());

                return weakCollectionChangedObservable.Scan(
                    data,
                    (list, args) =>
                    {
                        var changes = args;

                        switch (changes.Action)
                        {
                            case NotifyCollectionChangedAction.Add when changes.NewItems is not null:
                            {
                                if (changes.NewItems.Count == 1 && changes.NewItems[0] is T item)
                                    list.Insert(changes.NewStartingIndex, item);
                                else
                                    list.InsertRange(changes.NewItems.Cast<T>(), changes.NewStartingIndex);

                                break;
                            }

                            case NotifyCollectionChangedAction.Remove when changes.OldItems is not null:
                            {
                                if (changes.OldItems.Count == 1)
                                    list.RemoveAt(changes.OldStartingIndex);
                                else
                                    list.RemoveRange(changes.OldStartingIndex, changes.OldItems.Count);

                                break;
                            }

                            case NotifyCollectionChangedAction.Replace when changes.NewItems is not null &&
                                                                            changes.NewItems[0] is T replacedItem:
                                list[changes.NewStartingIndex] = replacedItem;
                                break;
                            case NotifyCollectionChangedAction.Reset:
                                list.Clear();
                                // list.AddRange(source);  <------ this is a major difference
                                break;
                            case NotifyCollectionChangedAction.Move:
                                list.Move(changes.OldStartingIndex, changes.NewStartingIndex);
                                break;
                        }

                        return list;
                    }).Select(list => list.CaptureChanges()).SubscribeSafe(observer);
            });
    }
}