using System.Reactive;
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
public partial class ReminderViewModel : EditableEntityViewModelBase<Reminder>, IValidatableViewModel
{
    [ObservableProperty] private bool _editing;
    [ObservableProperty] private string? _message;
    [ObservableProperty] private double? _threshold;

    public ReminderViewModel(Reminder entity, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(entity, entityCache, contexts)
    {
        Message = entity.Message;
        Threshold = entity.Threshold;
        StopEditingCommand = ReactiveCommand.Create(StopEditing, this.IsValid());

        this.ValidationRule(self => self.Message, message => !string.IsNullOrWhiteSpace(message), "Message is empty");
        this.ValidationRule(self => self.Threshold, threshold => threshold != null, "Threshold is empty");
    }

    public ReactiveCommand<Unit, Unit> StopEditingCommand { get; }

    public ValidationContext ValidationContext { get; } = new();

    public override void UpdateEntity()
    {
        // We ignore updating Alert here, since that will never get updated through this view model
        Entity.Message = Message ?? throw new InvalidOperationException(nameof(Message));
        Entity.Threshold = Threshold ?? throw new InvalidOperationException(nameof(Threshold));
    }

    public void StartEditing()
    {
        Editing = true;
    }

    public void StopEditing()
    {
        Editing = false;
    }
}