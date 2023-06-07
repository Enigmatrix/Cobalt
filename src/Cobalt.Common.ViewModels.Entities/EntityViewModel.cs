using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public abstract partial class EntityViewModel : ObservableObject
{
    public abstract IEntity Inner { get; }
    protected readonly IEntityViewModelCache Cache;
    protected EntityViewModel(IEntityViewModelCache cache)
    {
        Cache = cache;
    }

    [ObservableProperty] private long _id;
}

public abstract class EntityViewModel<T> : EntityViewModel
    where T : IEntity
{
    public override IEntity Inner => Entity;
    public T Entity = default!;

    protected EntityViewModel(IEntityViewModelCache cache) : base(cache)
    {
    }

    public virtual void InitializeWith(T entity)
    {
        Id = entity.Id;
        Entity = entity;
    }
}

public abstract class EditableEntityViewModel<T> : EntityViewModel<T>
    where T : IEntity
{
    protected readonly IDbContextFactory<CobaltContext> Conn;

    protected EditableEntityViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn) : base(cache)
    {
        Conn = conn;
    }

    public abstract void Save();
}
