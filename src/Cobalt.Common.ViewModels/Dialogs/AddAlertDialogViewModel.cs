using System.ComponentModel.DataAnnotations;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;
using ActionEntity = Cobalt.Common.Data.Entities.Action;

namespace Cobalt.Common.ViewModels.Dialogs;

public partial class AddAlertDialogViewModel : ObservableValidator
{
    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private ActionEntity? _action;

    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private TargetViewModel? _target;

    [CustomValidation(typeof(AddAlertDialogViewModel), nameof(ValidateUsageLimit))]
    [ObservableProperty]
    [NotifyDataErrorInfo]
    private TimeSpan _usageLimit;

    private TimeFrame _timeFrame = TimeFrame.Daily;

    public TimeFrame TimeFrame
    {
        get => _timeFrame;
        set
        {
            SetProperty(ref _timeFrame, value, true);
            ValidateProperty(UsageLimit, nameof(UsageLimit));
        }
    }

    public AddAlertDialogViewModel(IDbContextFactory<CobaltContext> conn)
    {
        ValidateAllProperties();
    }

    public static ValidationResult? ValidateUsageLimit(string _, ValidationContext context)
    {
        var instance = (AddAlertDialogViewModel)context.ObjectInstance;
        var maxUsageLimit = instance.TimeFrame switch
        {
            TimeFrame.Daily => TimeSpan.FromDays(1),
            TimeFrame.Weekly => TimeSpan.FromDays(7),
            TimeFrame.Monthly => TimeSpan.FromDays(31),
            _ => throw new ArgumentOutOfRangeException() // TODO
        };

        var invalid = instance.UsageLimit.Ticks <= 0 || instance.UsageLimit > maxUsageLimit;

        return invalid ? null : new ValidationResult("Usage Limit is longer than maximum duration for Time Frame");
    }

    [RelayCommand(CanExecute = nameof(CanAdd))]
    public void Add()
    {
    }

    public bool CanAdd()
    {
        return !HasErrors;
    }
}