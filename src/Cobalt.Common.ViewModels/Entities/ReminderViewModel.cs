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
///     ViewModel for the Reminder entity
/// </summary>
public interface IReminderViewModel : IValidatableViewModel, IReactiveObject
{
    public bool Editing { get; set; }
    public string Message { get; set; }
    public double Threshold { get; set; }

    public ReactiveCommand<Unit, Unit> StopEditingCommand { get; set; }
    public ReactiveCommand<Unit, Unit> StartEditingCommand { get; set; }

    public void Setup()
    {
        StartEditingCommand = ReactiveCommand.CreateFromTask(async () =>
        {
            await StartEditing();
            Editing = true;
        });
        StopEditingCommand = ReactiveCommand.CreateFromTask(async () =>
        {
            await StopEditing();
            Editing = false;
        }, this.IsValid());

        var messageValid = this.WhenAnyValue(self => self.Message, message => !string.IsNullOrWhiteSpace(message));
        var thresholdValid = this.WhenAnyValue(self => self.Threshold, threshold => !double.IsNaN(threshold));

        // Mark the properties as valid at start for the properties
        this.ValidationRule(self => self.Message, messageValid.Skip(1).StartWith(true), "Message is empty");
        this.ValidationRule(self => self.Threshold, thresholdValid.Skip(1).StartWith(true), "Threshold is empty");

        // Validate the whole model at the start
        this.ValidationRule(messageValid, "Message is empty");
        this.ValidationRule(thresholdValid, "Threshold is empty");
    }

    public Task StopEditing()
    {
        return Task.CompletedTask;
    }

    public Task StartEditing()
    {
        return Task.CompletedTask;
    }
}

/// <summary>
///     ViewModel for a new added Reminder Entity
/// </summary>
public partial class NewlyAddedReminderViewModel : ReactiveObservableObject, IReminderViewModel
{
    [ObservableProperty] private bool _editing;
    [ObservableProperty] private string? _message;
    [ObservableProperty] private double _threshold = double.NaN;

    public NewlyAddedReminderViewModel()
    {
        Editing = true;
        ((IReminderViewModel)this).Setup();
    }

    public ValidationContext ValidationContext { get; } = new();
    public ReactiveCommand<Unit, Unit> StopEditingCommand { get; set; } = default!;
    public ReactiveCommand<Unit, Unit> StartEditingCommand { get; set; } = default!;
}

/// <summary>
///     ViewModel for an already existing Reminder Entity
/// </summary>
public partial class ReminderViewModel : EditableEntityViewModelBase<Reminder>, IReminderViewModel
{
    [ObservableProperty] private bool _editing;
    [ObservableProperty] private string _message;
    [ObservableProperty] private double _threshold;

    public ReminderViewModel(Reminder entity, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(entity, entityCache,
        contexts)
    {
        Message = entity.Message;
        Threshold = entity.Threshold;
        Editing = false;

        ((IReminderViewModel)this).Setup();
    }

    public ReactiveCommand<Unit, Unit> StopEditingCommand { get; set; } = default!;
    public ReactiveCommand<Unit, Unit> StartEditingCommand { get; set; } = default!;


    public ValidationContext ValidationContext { get; } = new();

    public override void UpdateEntity()
    {
        // We ignore updating Alert here, since that will never get updated through this view model
        Entity.Message = Message ?? throw new InvalidOperationException(nameof(Message));
        Entity.Threshold = Threshold;
    }
}