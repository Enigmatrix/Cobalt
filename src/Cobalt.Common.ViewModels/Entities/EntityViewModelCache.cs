using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     Cache <see cref="EntityViewModelBase" /> according to their underlying <see cref="IEntity" />
/// </summary>
public interface IEntityViewModelCache
{
    /// <summary>
    ///     Convert the <see cref="Alert" /> to a cached <see cref="AlertViewModel" />
    /// </summary>
    AlertViewModel Alert(Alert alert);

    /// <summary>
    ///     Convert the <see cref="App" /> to a cached <see cref="AppViewModel" />
    /// </summary>
    AppViewModel App(App app);

    /// <summary>
    ///     Convert the <see cref="Tag" /> to a cached <see cref="TagViewModel" />
    /// </summary>
    TagViewModel Tag(Tag tag);
}

public record EntityViewModelCache(IDbContextFactory<QueryContext> Contexts) : IEntityViewModelCache
{
    public Dictionary<long, WeakReference<AppViewModel>> Apps { get; } = new();
    public Dictionary<long, WeakReference<TagViewModel>> Tags { get; } = new();
    public Dictionary<long, WeakReference<AlertViewModel>> Alerts { get; } = new();


    public TagViewModel Tag(Tag tag)
    {
        if (Tags.TryGetValue(tag.Id, out var v) && v.TryGetTarget(out var vm))
            return vm;

        var nvm = new TagViewModel(tag, this, Contexts);
        Tags[tag.Id] = new WeakReference<TagViewModel>(nvm);
        return nvm;
    }

    public AppViewModel App(App app)
    {
        if (Apps.TryGetValue(app.Id, out var v) && v.TryGetTarget(out var vm))
            return vm;

        var nvm = new AppViewModel(app, this, Contexts);
        Apps[app.Id] = new WeakReference<AppViewModel>(nvm);
        return nvm;
    }

    public AlertViewModel Alert(Alert alert)
    {
        if (Alerts.TryGetValue(alert.Id, out var v) && v.TryGetTarget(out var vm))
            return vm;

        var nvm = new AlertViewModel(alert, this, Contexts);
        Alerts[alert.Id] = new WeakReference<AlertViewModel>(nvm);
        return nvm;
    }
}