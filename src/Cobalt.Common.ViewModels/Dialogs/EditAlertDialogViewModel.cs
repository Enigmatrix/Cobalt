using System.Reactive;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using DynamicData;
using DynamicData.Binding;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Dialog ViewModel to edit existing Alerts
/// </summary>
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
        RemindersSource.AddRange(
            alert.Reminders.Select(reminder => new EditableReminderViewModel(this, reminder.Entity)));

        // This is validation context composition
        ValidationContext.Add(TriggerAction.ValidationContext);

        // - Check if any of the current Alert properties are not equal to the original AlertViewModel properties
        //    - Good thing about 2-level tracking is that we can directly compare the EntityViewModel of App/Tag
        // - Watch for TriggerAlert changes
        // - Check if any Reminder is dirty
        //    - For newly added Reminders, they are always considered dirty, so this works out.
        //    - If a newly added Reminder is then immediately removed, then the it's dirty-ness is removed, which is correct.
        // - Check if reminder count changes. This implies that elements might have been added/removed, specifically it catches removal.

        // watch if properties are changed
        var propsDirty = this.WhenAnyValue(
            self => self.ChooseTargetDialog!.Target,
            self => self.UsageLimit,
            self => self.TimeFrame,
            (target, usageLimit, timeFrame) =>
                target != originalTarget || usageLimit != alert.UsageLimit || timeFrame != alert.TimeFrame);
        // combine with TriggerActionViewModel
        var triggerActionDirty = TriggerAction.WhenAnyPropertyChanged().Select(triggerAction =>
            triggerAction!.Inner != alert.TriggerAction).StartWith(false);
        // combine with Reminders
        var remindersDirty = RemindersSource.Connect()
            .AddKey(reminder => reminder)
            .TrueForAny(reminder => reminder.WhenAnyValue(self => self.IsDirty), dirty => dirty)
            .StartWith(false);
        // watch if reminder count changes
        var remindersCountChanged = RemindersSource.CountChanged.Select(count => count != alert.Reminders.Count);

        propsDirty.CombineLatest(triggerActionDirty, remindersDirty, remindersCountChanged,
                (a, b, c, d) => a || b || c || d)
            .BindTo(this, self => self.IsDirty);
    }


    public override string Title => "Edit Alert";
    public override string PrimaryButtonText => "Save";
    public override string CloseButtonText => "Discard";

    public override ReactiveCommand<Unit, Unit> PrimaryButtonCommand =>
        ReactiveCommand.CreateFromTask(SaveAlertAsync,
            this.IsValid()
                .CombineLatest(this.WhenAnyValue(self => self.IsDirty),
                    (valid, dirty) => valid && dirty));

    public async Task SaveAlertAsync()
    {
        await using var context = await Contexts.CreateDbContextAsync();
        var alert = _alert.Entity;
        alert.App = null;
        alert.Tag = null;
        context.Attach(alert);

        alert.TimeFrame = TimeFrame!.Value;
        alert.TriggerAction = TriggerAction!.Inner!;
        alert.UsageLimit = UsageLimit!.Value;

        switch (ChooseTargetDialog!.Target)
        {
            case AppViewModel app:
                alert.App = app.Entity;
                context.Attach(alert.App);
                break;
            case TagViewModel tag:
                alert.Tag = tag.Entity;
                context.Attach(alert.Tag);
                break;
        }

        await context.UpdateAlertAsync(alert);

        var existingReminderVms = Reminders.Where(reminderVm => reminderVm.Reminder != null).ToList();

        // Delete existing reminders that are not in the final list
        foreach (var reminderVm in _alert.Reminders)
            if (existingReminderVms.Select(vm => vm.Reminder).All(reminder =>
                    reminder!.Id != reminderVm.Entity.Id && reminder.Id != reminderVm.Entity.Id))
                context.Remove(reminderVm.Entity);

        // Update existing reminders that are in the final list and dirty
        foreach (var reminderVm in existingReminderVms.Where(vm => vm.IsDirty))
        {
            var reminder = reminderVm.Reminder!;
            reminder.Message = reminderVm.Message!;
            reminder.Threshold = reminderVm.Threshold;
            await context.UpdateReminderAsync(reminder);
        }

        // Add new reminders
        foreach (var reminderVm in Reminders.Where(reminderVm => reminderVm.Reminder == null))
            context.Add(new Reminder
            {
                Alert = alert,
                Threshold = reminderVm.Threshold,
                Message = reminderVm.Message!,
            });

        await context.SaveChangesAsync();

        _alert.InitializeWith(alert);
        Result = _alert;
    }
}