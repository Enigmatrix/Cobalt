﻿using System.Runtime.CompilerServices;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.ViewModels.Entities;

public class EntityViewModelCache
{
    private readonly CobaltContext _ctx;

    public EntityViewModelCache(CobaltContext ctx)
    {
        _ctx = ctx;
    }

    public ConditionalWeakTable<App, AppViewModel> Apps { get; } = new();

    public ConditionalWeakTable<Tag, TagViewModel> Tags { get; } = new();

    public ConditionalWeakTable<Alert, AlertViewModel> Alerts { get; } = new();

    public TagViewModel GetForTag(Tag tag)
    {
        return Tags.GetValue(tag, t =>
        {
            var tvm = new TagViewModel(_ctx, this);
            tvm.InitializeEntity(t);
            return tvm;
        });
    }

    public AppViewModel GetForApp(App app)
    {
        return Apps.GetValue(app, a =>
        {
            var avm = new AppViewModel(_ctx, this);
            avm.InitializeEntity(a);
            return avm;
        });
    }

    public AlertViewModel GetForAlert(Alert alert)
    {
        return Alerts.GetValue(alert, a =>
        {
            var avm = new AlertViewModel(_ctx, this);
            avm.InitializeEntity(a);
            return avm;
        });
    }
}