using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the Alert Entity
/// </summary>
public partial class AlertViewModel : EditableEntityViewModelBase<Alert>
{
    [ObservableProperty] private AppViewModel? _app;
    [ObservableProperty] private List<ReminderViewModel> _reminders = default!;
    [ObservableProperty] private TagViewModel? _tag;
    [ObservableProperty] private TimeFrame _timeFrame;
    [ObservableProperty] private TriggerAction _triggerAction = default!;
    [ObservableProperty] private TimeSpan _usageLimit;

    public AlertViewModel(Alert entity, IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(entity, entityCache, contexts)
    {
        InitializeWith(entity);
    }

    public void InitializeWith(Alert entity)
    {
        TimeFrame = entity.TimeFrame;
        UsageLimit = entity.UsageLimit;
        TriggerAction = entity.TriggerAction;
        if (entity.App != null)
        {
            App = EntityCache.App(entity.App);
            Tag = null;
        }
        else if (entity.Tag != null)
        {
            App = null;
            Tag = EntityCache.Tag(entity.Tag);
        }

        Reminders = entity.Reminders.Select(EntityCache.Reminder).ToList();
    }

    public override void UpdateEntity()
    {
        Entity.TimeFrame = TimeFrame;
        Entity.UsageLimit = UsageLimit;
        Entity.TriggerAction = TriggerAction;
        if (App != null) Entity.App = App.Entity;
        else if (Tag != null) Entity.Tag = Tag.Entity;
        Entity.Reminders.Clear();
        Entity.Reminders.AddRange(Reminders.Select(reminder => reminder.Entity));
    }
}