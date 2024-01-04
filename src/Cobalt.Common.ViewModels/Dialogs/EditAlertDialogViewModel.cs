using System.Reactive;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
using DynamicData;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Dialogs;

public class EditAlertDialogViewModel : AlertDialogViewModelBase
{
    private readonly AlertViewModel _alert;

    public EditAlertDialogViewModel(AlertViewModel alert, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(
        entityCache, contexts)
    {
        _alert = alert;
        ChooseTargetDialog =
            new ChooseTargetDialogViewModel(EntityCache, Contexts, (EntityViewModelBase?)alert.App ?? alert.Tag!);
        UsageLimit = alert.UsageLimit;
        TimeFrame = alert.TimeFrame;
        TriggerAction = new TriggerActionViewModel(alert.TriggerAction);
        RemindersSource.AddRange(alert.Reminders.Select(reminder => new EditableReminderViewModel(reminder.Entity)));

        // This is validation context composition
        ValidationContext.Add(TriggerAction.ValidationContext);
    }


    public override string Title => "Edit Alert";


    public override ReactiveCommand<Unit, Unit> PrimaryButtonCommand =>
        ReactiveCommand.CreateFromTask(SaveAlertAsync, this.IsValid()); // TODO valid + isdirty

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