﻿using System.Reactive.Disposables;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Dialog ViewModel to Add Alert
/// </summary>
public partial class AddAlertDialogViewModel : DialogViewModelBase<AlertViewModel>
{
    private readonly IEntityViewModelCache _entityCache;
    [ObservableProperty] private AppViewModel? _selectedApp;
    [ObservableProperty] private TagViewModel? _selectedTag;
    [ObservableProperty] private EntityViewModelBase? _selectedTarget;
    [ObservableProperty] private string _targetSearch = "";
    [ObservableProperty] private TimeFrame _timeFrame;
    [ObservableProperty] private TriggerAction? _triggerAction; // TODO init this?

    // TODO find some validation library!!!!
    [ObservableProperty] private TimeSpan _usageLimit;

    public AddAlertDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        _entityCache = entityCache;
        var searches = this.WhenAnyValue(self => self.TargetSearch);

        // TODO commonalize the search expression 
        // TODO use some case-ignore string search instead of tolower

        Apps = Query(searches,
            async (context, search) => (await context.Apps.Where(app =>
                app.Name.ToLower().Contains(search.ToLower()) ||
                app.Description.ToLower().Contains(search.ToLower()) ||
                app.Company.ToLower().Contains(search.ToLower())
            ).ToListAsync()).Select(_entityCache.App).ToList());

        Tags = Query(searches,
            async (context, search) => (await context.Tags.Where(tag =>
                tag.Name.ToLower().Contains(search.ToLower())
            ).ToListAsync()).Select(_entityCache.Tag).ToList());

        this.WhenActivated(dis =>
        {
            TargetSearch = "";
            Apps.Refresh();
            Tags.Refresh();


            // Reset SelectedTarget based on the two selection properties, SelectedApp and SelectedTag
            this.WhenAnyValue(self => self.SelectedApp)
                .Where(self => self != null)
                .Subscribe(self =>
                {
                    SelectedTag = null;
                    SelectedTarget = self!;
                })
                .DisposeWith(dis);

            this.WhenAnyValue(self => self.SelectedTag)
                .Where(self => self != null)
                .Subscribe(self =>
                {
                    SelectedApp = null;
                    SelectedTarget = self!;
                })
                .DisposeWith(dis);

            this.WhenAnyValue(self => self.SelectedTag, self => self.SelectedApp,
                    (tag, app) => tag == null && app == null)
                .DistinctUntilChanged()
                .Subscribe(_ => SelectedTarget = null)
                .DisposeWith(dis);
        });
    }

    public Query<string, List<AppViewModel>> Apps { get; }
    public Query<string, List<TagViewModel>> Tags { get; }

    public override string Title => "Add Alert";

    public override async Task<AlertViewModel> GetResultAsync()
    {
        var alert = new Alert
        {
            Guid = Guid.NewGuid(),
            Version = 1,
            TimeFrame = TimeFrame,
            TriggerAction = TriggerAction,
            UsageLimit = UsageLimit
        };
        switch (SelectedTarget)
        {
            case AppViewModel app:
                alert.App = app.Entity;
                break;
            case TagViewModel tag:
                alert.Tag = tag.Entity;
                break;
        }

        await using var context = await Contexts.CreateDbContextAsync();
        await context.AddAsync(alert);
        await context.SaveChangesAsync();

        return new AlertViewModel(alert, _entityCache, Contexts);
    }
}