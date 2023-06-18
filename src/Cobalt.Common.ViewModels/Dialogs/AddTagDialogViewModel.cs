using System.Collections.ObjectModel;
using System.ComponentModel.DataAnnotations;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Dialogs;

public partial class AddTagDialogViewModel : ObservableValidator
{
    private static readonly Random Random = new();

    // ref: https://github.com/catppuccin/catppuccin
    // Mocha colors
    private static readonly string[] Colors =
    {
        "#f5e0dc", "#f2cdcd", "#f5c2e7", "#cba6f7", "#f38ba8", "#eba0ac", "#fab387", "#f9e2af",
        "#a6e3a1", "#94e2d5", "#89dceb", "#74c7ec", "#89b4fa", "#b4befe",
        "#cdd6f4" // Text
    };

    private readonly IEntityViewModelCache _cache;
    private readonly IDbContextFactory<CobaltContext> _conn;

    [ObservableProperty] private string _color;

    [Required] [NotifyDataErrorInfo] [ObservableProperty]
    private string? _name;

    public AddTagDialogViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn)
    {
        _conn = conn;
        _cache = cache;

        _color = RandomColor();
        ValidateAllProperties();
    }

    public ObservableCollection<AppViewModel> Apps { get; } = new();

    public ObservableCollection<AppViewModel> AllApps
    {
        get
        {
            using var db = _conn.CreateDbContext();
            return new ObservableCollection<AppViewModel>(db.Apps.ToList().Select(app => _cache.App(app)).Except(Apps));
        }
    }

    public Func<string, CancellationToken, Task<IEnumerable<object>>> SearchAppsHandler => SearchApps;

    public async Task<IEnumerable<object>> SearchApps(string query, CancellationToken cancelToken)
    {
        await using var db = await _conn.CreateDbContextAsync(cancelToken);
        // TODO fts in the future
        var apps = await db.Apps
            .Where(app => app.Name.ToLower().Contains(query.ToLower()))
            .ToListAsync(cancelToken);
        return apps.Select(app => _cache.App(app)).Except(Apps).Select(x => (object)x);
    }


    // TODO set this as the dialog's result
    [RelayCommand(CanExecute = nameof(CanAdd))]
    public void Add()
    {
        using var db = _conn.CreateDbContext();

        db.Attach(new Tag
        {
            Name = Name ?? throw new NullReferenceException(nameof(Name)),
            Color = Color,
            Apps = Apps.Select(app => new App { Id = app.Id }).ToList()
        });

        db.SaveChanges();
    }


    [RelayCommand]
    public void RemoveApp(AppViewModel app)
    {
        Apps.Remove(app);
    }

    [RelayCommand]
    public void AddApp(AppViewModel app)
    {
        Apps.Add(app);
    }

    public bool CanAdd()
    {
        return !HasErrors;
    }

    private static string RandomColor()
    {
        return Colors[Random.Next(Colors.Length)];
    }
}