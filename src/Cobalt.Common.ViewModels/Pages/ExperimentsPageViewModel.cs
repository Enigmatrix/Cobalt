using System.Diagnostics;
using Cobalt.Common.Data;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Experiments Page
/// </summary>
/// <remarks>
///     This class does not exist during production.
/// </remarks>
public partial class ExperimentsPageViewModel : PageViewModelBase, IValidatableViewModel
{
    [ObservableProperty] private TimeSpan? _lmaoDuration = TimeSpan.FromHours(9);

    public ExperimentsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
        this.ValidationRule(self => self.LmaoDuration, dur => (dur ?? TimeSpan.Zero) < TimeSpan.FromHours(5), "Too long dick!");
        this.WhenAnyValue(self => self.LmaoDuration).Subscribe(s => Debug.WriteLine($"dur: {s}"));
    }

    public override string Name => "Experiments";

    [RelayCommand]
    public async Task UpdateAllUsageEndsAsync()
    {
        await using var context = await Contexts.CreateDbContextAsync();
        await context.UpdateAllUsageEndsAsync(DateTime.Now);
    }

    public ValidationContext ValidationContext { get; } = new();
}