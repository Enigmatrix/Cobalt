using System.Reactive;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using DynamicData;
using DynamicData.Binding;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Dialogs;

public partial class EditAlertDialogViewModel : AlertDialogViewModelBase
{
    private readonly AlertViewModel _alert;
    [ObservableProperty] private bool _isDirty;

    public EditAlertDialogViewModel(AlertViewModel alert, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(
        entityCache, contexts)
    {
        _alert = alert;
        var originalTarget = (EntityViewModelBase?)alert.App ?? alert.Tag!;
        ChooseTargetDialog =
            new ChooseTargetDialogViewModel(EntityCache, Contexts, originalTarget);
        UsageLimit = alert.UsageLimit;
        TimeFrame = alert.TimeFrame;
        TriggerAction = new TriggerActionViewModel(alert.TriggerAction);
        RemindersSource.AddRange(alert.Reminders.Select(reminder => new EditableReminderViewModel(reminder.Entity)));

        // This is validation context composition
        ValidationContext.Add(TriggerAction.ValidationContext);

        var propsDirty = this.WhenAnyValue(
            self => self.ChooseTargetDialog!.Target,
            self => self.UsageLimit,
            self => self.TimeFrame,
            (target, usageLimit, timeFrame) =>
                target != originalTarget || usageLimit != alert.UsageLimit || timeFrame != alert.TimeFrame);
        // combine with TriggerActionViewModel
        var triggerActionDirty = TriggerAction.WhenAnyPropertyChanged().Select(triggerAction =>
            triggerAction!.ToTriggerAction() != alert.TriggerAction).StartWith(false);
        // combine with Reminders
        var remindersDirty = RemindersSource.Connect()
            .AddKey(reminder => reminder)
            .TrueForAny(reminder => reminder.WhenAnyValue(self => self.IsDirty), dirty => dirty)
            .StartWith(false);
        var remindersCountChanged = RemindersSource.CountChanged.Select(count => count != alert.Reminders.Count);

        propsDirty.CombineLatest(triggerActionDirty, remindersDirty, remindersCountChanged,
                (a, b, c, d) => a || b || c || d)
            .BindTo(this, self => self.IsDirty);
    }


    public override string Title => "Edit Alert";
    public override string PrimaryButtonText => "Save";
    public override string CloseButtonText => "Discard";

    // TODO IsDirty tracking:
    // - Check if any of the current Alert properties are not equal to the original AlertViewModel properties
    //    - Good thing about 2-level tracking is that we can directly compare the EntityViewModel of App/Tag
    // - Check if any Reminder is dirty
    //    - For newly added Reminders, they are always considered dirty, so this works out.
    //    - If a newly added Reminder is then immediately removed, then the it's dirty-ness is removed, which is correct.

    // TODO if we switch from kill -> dimduration, the first duration entered in dimduration does not trigger validation - the custom validation shit needs to be fixed!

    public override ReactiveCommand<Unit, Unit> PrimaryButtonCommand =>
        ReactiveCommand.CreateFromTask(SaveAlertAsync,
            this.IsValid()
                .CombineLatest(this.WhenAnyValue(self => self.IsDirty),
                    (valid, dirty) => valid && dirty));

    public async Task SaveAlertAsync()
    {
        /*var alert = new Alert
        {
            Guid = Guid.NewGuid(),
            Version = 1,
            TimeFrame = TimeFrame!.Value,
            TriggerAction = TriggerAction!.ToTriggerAction(),
            UsageLimit = UsageLimit!.Value
        };
        alert.Reminders.AddRange(Reminders.Select(reminder => new Reminder
        {
            Guid = Guid.NewGuid(),
            Version = 1,
            Message = reminder.Message!,
            Threshold = reminder.Threshold,
            Alert = alert
        }));
        switch (ChooseTargetDialog!.Target)
        {
            case AppViewModel app:
                alert.App = app.Entity;
                break;
            case TagViewModel tag:
                alert.Tag = tag.Entity;
                break;
        }

        await using var context = await Contexts.CreateDbContextAsync();
        // Attach everything except reminders, which are instead in the added state
        context.Attach(alert);
        context.AddRange(alert.Reminders);
        await context.AddAsync(alert);
        await context.SaveChangesAsync();*/

        Result = _alert;
    }
}