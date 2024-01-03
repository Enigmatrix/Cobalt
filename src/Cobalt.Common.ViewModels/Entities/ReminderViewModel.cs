using System.Reactive;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the Reminder Entity
/// </summary>
public partial class ReminderViewModel : EditableEntityViewModelBase<Reminder>, IActivatableViewModel,
    IValidatableViewModel
{
    private static readonly Reminder NullReminder = new()
    {
        Alert = default!,
        Threshold = 0,
        Message = default!,
        Version = 0,
        Guid = default
    };

    [ObservableProperty] private bool _editing;
    [ObservableProperty] private string? _message;
    [ObservableProperty] private double _threshold = double.NaN;

    public ReminderViewModel(Reminder? entity, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts, bool editing = false) : base(entity ?? NullReminder, entityCache,
        contexts)
    {
        if (entity != null)
        {
            Message = entity.Message;
            Threshold = entity.Threshold;
        }

        Editing = editing;
        StopEditingCommand = ReactiveCommand.Create(StopEditing, this.IsValid());

        var messageValid = this.WhenAnyValue(self => self.Message, message => !string.IsNullOrWhiteSpace(message));
        var thresholdValid = this.WhenAnyValue(self => self.Threshold, threshold => !double.IsNaN(threshold));

        // Mark the properties as valid at start for the properties
        this.ValidationRule(self => self.Message, messageValid.Skip(1).StartWith(true), "Message is empty");
        this.ValidationRule(self => self.Threshold, thresholdValid.Skip(1).StartWith(true), "Threshold is empty");

        // Validate the whole model at the start
        this.ValidationRule(messageValid, "Message is empty");
        this.ValidationRule(thresholdValid, "Threshold is empty");
    }

    public ReactiveCommand<Unit, Unit> StopEditingCommand { get; }


    public ViewModelActivator Activator { get; } = new();
    public ValidationContext ValidationContext { get; } = new();

    public override void UpdateEntity()
    {
        // We ignore updating Alert here, since that will never get updated through this view model
        Entity.Message = Message ?? throw new InvalidOperationException(nameof(Message));
        Entity.Threshold = Threshold;
    }

    public void StartEditing()
    {
        Editing = true;
    }

    private void StopEditing()
    {
        Editing = false;
    }
}