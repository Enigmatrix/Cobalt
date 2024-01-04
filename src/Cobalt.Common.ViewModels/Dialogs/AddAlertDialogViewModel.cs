using System.Reactive;
using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Entities;
using DynamicData;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Dialogs;

public class AddAlertDialogViewModel : AlertDialogViewModelBase
{
    public AddAlertDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) : base(
        entityCache, contexts)
    {
        this.WhenActivated((CompositeDisposable dis) =>
        {
            RemindersSource.Clear();
            Reminders.Clear(); // since we dispose of the Bind, we need to clean this ourselves
            UsageLimit = null;
            TimeFrame = null;
            TriggerAction.Clear();
            ChooseTargetDialog.Clear();
        });
    }

    public override string Title => "Add Alert";


    public override ReactiveCommand<Unit, Unit> PrimaryButtonCommand =>
        ReactiveCommand.CreateFromTask(AddAlertAsync, this.IsValid());

    public async Task AddAlertAsync()
    {
        var alert = new Alert
        {
            Guid = Guid.NewGuid(),
            Version = 1,
            TimeFrame = TimeFrame!.Value,
            TriggerAction = TriggerAction.ToTriggerAction(),
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
        switch (ChooseTargetDialog.Target)
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
        await context.SaveChangesAsync();

        Result = new AlertViewModel(alert, EntityCache, Contexts);
    }
}