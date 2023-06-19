using System.Collections.ObjectModel;
using System.ComponentModel.DataAnnotations;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Utils;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Interactions;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;
using ActionEntity = Cobalt.Common.Data.Entities.Action;

namespace Cobalt.Common.ViewModels.Dialogs;

public partial class AddAlertDialogViewModel : DialogViewModelBase<Unit, AlertViewModel>
{
    private readonly Func<AddTagDialogViewModel> _addTagDialogFactory;
    private readonly IEntityViewModelCache _cache;

    private readonly IDbContextFactory<CobaltContext> _conn;

    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private ActionEntity? _action;

    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private TargetViewModel? _target;

    private TimeFrame _timeFrame = TimeFrame.Daily;

    [CustomValidation(typeof(AddAlertDialogViewModel), nameof(ValidateUsageLimit))]
    [ObservableProperty]
    [NotifyDataErrorInfo]
    private TimeSpan _usageLimit;

    public AddAlertDialogViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn,
        Func<AddTagDialogViewModel> addTagDialogFactory)
    {
        _conn = conn;
        _cache = cache;

        _addTagDialogFactory = addTagDialogFactory;
        AddTagInteraction = new Interaction<AddTagDialogViewModel, TagViewModel>();
        ValidateAllProperties();
    }

    public Interaction<AddTagDialogViewModel, TagViewModel> AddTagInteraction { get; }

    public TimeFrame TimeFrame
    {
        get => _timeFrame;
        set
        {
            SetProperty(ref _timeFrame, value, true);
            OnPropertyChanged(nameof(MaxUsageLimit));
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

    public TimeSpan MaxUsageLimit =>
        TimeFrame switch
        {
            TimeFrame.Daily => TimeSpan.FromDays(1),
            TimeFrame.Weekly => TimeSpan.FromDays(7),
            TimeFrame.Monthly => TimeSpan.FromDays(31),
            _ => throw new DiscriminatedUnionException<TimeFrame>(nameof(TimeFrame))
        };

    public static ValidationResult? ValidateUsageLimit(string _, ValidationContext context)
    {
        var instance = (AddAlertDialogViewModel)context.ObjectInstance;

        var invalid = instance.UsageLimit.Ticks <= 0 || instance.UsageLimit > instance.MaxUsageLimit;

        return invalid ? null : new ValidationResult("Usage Limit is longer than maximum duration for Time Frame");
    }

    [RelayCommand]
    public async Task AddTagAndSetTarget()
    {
        var tag = await AddTagInteraction.Handle(_addTagDialogFactory());
        if (tag == null) return;
        Target = new TargetViewModel.TagTarget(tag);
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