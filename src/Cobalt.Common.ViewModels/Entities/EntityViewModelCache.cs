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
    public ConcurrentDictionary<long, WeakReference<AppViewModel>> Apps { get; } = new();
    public ConcurrentDictionary<long, WeakReference<TagViewModel>> Tags { get; } = new();
    public ConcurrentDictionary<long, WeakReference<AlertViewModel>> Alerts { get; } = new();
    public ConcurrentDictionary<long, WeakReference<ReminderViewModel>> Reminders { get; } = new();


    public TagViewModel Tag(Tag tag)
    {
        while (true)
        {
            var refvm = Tags.AddOrUpdate(tag.Id,
                (id, tagParam) => new WeakReference<TagViewModel>(new TagViewModel(tagParam, this, Contexts)),
                (id, prev, tagParam) =>
                {
                    if (!prev.TryGetTarget(out _))
                        prev = new WeakReference<TagViewModel>(new TagViewModel(tagParam, this, Contexts));
                    return prev;
                }, tag);

            if (!refvm.TryGetTarget(out var vm)) continue;

            return vm;
        }
    }

    public AppViewModel App(App app)
    {
        while (true)
        {
            var refvm = Apps.AddOrUpdate(app.Id,
                (id, appParam) => new WeakReference<AppViewModel>(new AppViewModel(appParam, this, Contexts)),
                (id, prev, appParam) =>
                {
                    if (!prev.TryGetTarget(out _))
                        prev = new WeakReference<AppViewModel>(new AppViewModel(appParam, this, Contexts));
                    return prev;
                }, app);

            if (!refvm.TryGetTarget(out var vm))
                continue;
            return vm;
        }
    }

    public AlertViewModel Alert(Alert alert)
    {
        while (true)
        {
            var refvm = Alerts.AddOrUpdate(alert.Id,
                (id, alertParam) => new WeakReference<AlertViewModel>(new AlertViewModel(alertParam, this, Contexts)),
                (id, prev, alertParam) =>
                {
                    if (!prev.TryGetTarget(out _))
                        prev = new WeakReference<AlertViewModel>(new AlertViewModel(alertParam, this, Contexts));
                    return prev;
                }, alert);

            if (!refvm.TryGetTarget(out var vm)) continue;
            return vm;
        }
    }

    public ReminderViewModel Reminder(Reminder reminder)
    {
        while (true)
        {
            var refvm = Reminders.AddOrUpdate(reminder.Id,
                (id, reminderParam) =>
                    new WeakReference<ReminderViewModel>(new ReminderViewModel(reminderParam, this, Contexts)),
                (id, prev, reminderParam) =>
                {
                    if (!prev.TryGetTarget(out _))
                        prev = new WeakReference<ReminderViewModel>(
                            new ReminderViewModel(reminderParam, this, Contexts));
                    return prev;
                }, reminder);

            if (!refvm.TryGetTarget(out var vm)) continue;
            return vm;
        }
    }
}