using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public interface IEntityViewModelCache
{
    AlertViewModel Alert(Alert alert);
    AppViewModel App(App app);
    TagViewModel Tag(Tag tag);
}

public class EntityViewModelCache : IEntityViewModelCache
{
    private readonly IDbContextFactory<CobaltContext> _conn;

    public EntityViewModelCache(IDbContextFactory<CobaltContext> conn)
    {
        _conn = conn;
    }

    public Dictionary<long, WeakReference<AppViewModel>> Apps { get; } = new();
    public Dictionary<long, WeakReference<TagViewModel>> Tags { get; } = new();
    public Dictionary<long, WeakReference<AlertViewModel>> Alerts { get; } = new();


    public TagViewModel Tag(Tag tag)
    {
        if (Tags.TryGetValue(tag.Id, out var v) && v.TryGetTarget(out var vm))
            return vm;

        var nvm = new TagViewModel(this, _conn);
        nvm.InitializeWith(tag);
        Tags[tag.Id] = new WeakReference<TagViewModel>(nvm);
        return nvm;
    }

    public AppViewModel App(App app)
    {
        if (Apps.TryGetValue(app.Id, out var v) && v.TryGetTarget(out var vm))
            return vm;

        var nvm = new AppViewModel(this, _conn);
        nvm.InitializeWith(app);
        Apps[app.Id] = new WeakReference<AppViewModel>(nvm);
        return nvm;
    }

    public AlertViewModel Alert(Alert alert)
    {
        if (Alerts.TryGetValue(alert.Id, out var v) && v.TryGetTarget(out var vm))
            return vm;

        var nvm = new AlertViewModel(this, _conn);
        nvm.InitializeWith(alert);
        Alerts[alert.Id] = new WeakReference<AlertViewModel>(nvm);
        return nvm;
    }
}