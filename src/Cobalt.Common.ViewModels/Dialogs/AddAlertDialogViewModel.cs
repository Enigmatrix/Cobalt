using System.Collections.ObjectModel;
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
    private readonly IEntityViewModelCache _cache;

    private readonly IDbContextFactory<CobaltContext> _conn;

    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private ActionEntity? _action;

    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private Target? _target;

    private TimeFrame _timeFrame = TimeFrame.Daily;

    [CustomValidation(typeof(AddAlertDialogViewModel), nameof(ValidateUsageLimit))]
    [ObservableProperty]
    [NotifyDataErrorInfo]
    private TimeSpan _usageLimit;

    public AddAlertDialogViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn)
    {
        _conn = conn;
        _cache = cache;
        ValidateAllProperties();
    }

    public TimeFrame TimeFrame
    {
        get => _timeFrame;
        set
        {
            SetProperty(ref _timeFrame, value, true);
            ValidateProperty(UsageLimit, nameof(UsageLimit));
        }
    }

    public ObservableCollection<AppViewModel> AllApps
    {
        get
        {
            using var db = _conn.CreateDbContext();
            return new ObservableCollection<AppViewModel>(db.Apps.ToList().Select(app => _cache.App(app)));
        }
    }

    public ObservableCollection<TagViewModel> AllTags
    {
        get
        {
            using var db = _conn.CreateDbContext();
            return new ObservableCollection<TagViewModel>(db.Tags.ToList().Select(tag => _cache.Tag(tag)));
        }
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