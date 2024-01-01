using System.Collections.Concurrent;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     Cache <see cref="EntityViewModelBase" /> according to their underlying <see cref="IEntity" />
/// </summary>
public interface IEntityViewModelCache
{
    /// <summary>
    ///     Convert the <see cref="Alert" /> to a cached <see cref="AlertViewModel" />
    /// </summary>
    AlertViewModel Alert(Alert alert);

    /// <summary>
    ///     Convert the <see cref="App" /> to a cached <see cref="AppViewModel" />
    /// </summary>
    AppViewModel App(App app);

    /// <summary>
    ///     Convert the <see cref="Tag" /> to a cached <see cref="TagViewModel" />
    /// </summary>
    TagViewModel Tag(Tag tag);

    /// <summary>
    ///     Convert the <see cref="Reminder" /> to a cached <see cref="ReminderViewModel" />
    /// </summary>
    ReminderViewModel Reminder(Reminder reminder);
}

public record EntityViewModelCache(IDbContextFactory<QueryContext> Contexts) : IEntityViewModelCache
{
    public WeakConcurrentDictionary<long, AppViewModel> Apps { get; } = new();
    public WeakConcurrentDictionary<long, TagViewModel> Tags { get; } = new();
    public WeakConcurrentDictionary<long, AlertViewModel> Alerts { get; } = new();
    public WeakConcurrentDictionary<long, ReminderViewModel> Reminders { get; } = new();


    public TagViewModel Tag(Tag tag)
    {
        return Tags.Fetch(tag.Id, tag, entity => new TagViewModel(entity, this, Contexts));
    }

    public AppViewModel App(App app)
    {
        return Apps.Fetch(app.Id, app, entity => new AppViewModel(entity, this, Contexts));
    }

    public AlertViewModel Alert(Alert alert)
    {
        return Alerts.Fetch(alert.Id, alert, entity => new AlertViewModel(entity, this, Contexts));
    }

    public ReminderViewModel Reminder(Reminder reminder)
    {
        return Reminders.Fetch(reminder.Id, reminder, entity => new ReminderViewModel(entity, this, Contexts));
    }
}

public class WeakConcurrentDictionary<TKey, TValue>
    where TKey : notnull
    where TValue : class
{
    private readonly ConcurrentDictionary<TKey, WeakReference<TValue>> _inner = new();

    public TValue Fetch<TArg>(TKey key, TArg arg, Func<TArg, TValue> create)
    {
        if (_inner.TryGetValue(key, out var wv) && wv.TryGetTarget(out var v)) return v;

        TValue? strongRef = null;

        TValue CreateRef()
        {
            return strongRef = create(arg);
        }

        void SetRef(TValue value)
        {
            strongRef = value;
        }

        _inner.AddOrUpdate(key, static (_, ops) => new WeakReference<TValue>(ops.Item1()),
            static (_, prev, ops) =>
            {
                if (prev.TryGetTarget(out var v))
                    ops.Item2(v);
                else
                    prev = new WeakReference<TValue>(ops.Item1());
                return prev;
            }, ValueTuple.Create(CreateRef, SetRef));

        return strongRef!;
    }
}