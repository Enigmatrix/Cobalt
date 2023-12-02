using Cobalt.Common.Data;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public abstract partial class EntityViewModelBase : ViewModelBase
{
    [ObservableProperty] private long _id;

    public abstract IEntity Inner { get; }
}

public abstract class EntityViewModelBase<T> : EntityViewModelBase where T : IEntity
{
    protected readonly T Entity;

    protected EntityViewModelBase(T entity)
    {
        Entity = entity;
        Id = Entity.Id;
    }

    public override IEntity Inner => Entity;
}

public abstract class EditableEntityViewModelBase<T> : EntityViewModelBase<T> where T : IEntity
{
    protected readonly IDbContextFactory<QueryContext> Contexts;

    protected EditableEntityViewModelBase(T entity, IDbContextFactory<QueryContext> contexts) : base(entity)
    {
        Contexts = contexts;
    }

    public abstract void UpdateEntity();

    public virtual void Save()
    {
        using var ctx = Contexts.CreateDbContext();
        UpdateEntity();
        ctx.Update(Entity);
        ctx.SaveChanges();
    }
}