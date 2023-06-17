using System.Runtime.CompilerServices;
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

    public ConditionalWeakTable<App, AppViewModel> Apps { get; } = new();

    public ConditionalWeakTable<Tag, TagViewModel> Tags { get; } = new();

    public ConditionalWeakTable<Alert, AlertViewModel> Alerts { get; } = new();

    public TagViewModel Tag(Tag tag)
    {
        return Tags.GetValue(tag, t =>
        {
            var tvm = new TagViewModel(this, _conn);
            tvm.InitializeWith(t);
            return tvm;
        });
    }

    public AppViewModel App(App app)
    {
        return Apps.GetValue(app, t =>
        {
            var tvm = new AppViewModel(this, _conn);
            tvm.InitializeWith(t);
            return tvm;
        });
    }

    public AlertViewModel Alert(Alert alert)
    {
        return Alerts.GetValue(alert, t =>
        {
            var tvm = new AlertViewModel(this, _conn);
            tvm.InitializeWith(t);
            return tvm;
        });
    }
}