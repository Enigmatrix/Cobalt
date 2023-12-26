using Cobalt.Common.Data;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     Base class for all Entity ViewModels
/// </summary>
public abstract partial class EntityViewModelBase : ViewModelBase
{
    [ObservableProperty] private long _id;

    protected EntityViewModelBase(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public abstract IEntity Inner { get; }
}

/// <summary>
///     Base class for all <see cref="IEntity" /> ViewModels that is generic over the Entity
/// </summary>
public abstract class EntityViewModelBase<T> : EntityViewModelBase where T : IEntity
{
    public readonly T Entity;
    protected readonly IEntityViewModelCache EntityCache;

    protected EntityViewModelBase(T entity, IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts)
        : base(contexts)
    {
        Entity = entity;
        EntityCache = entityCache;
        Id = Entity.Id;
    }

    public override IEntity Inner => Entity;
}

/// <summary>
///     Base class for all <see cref="IEntity" /> ViewModels that is generic over the Entity and editable and saveable.
/// </summary>
public abstract class EditableEntityViewModelBase<T> : EntityViewModelBase<T> where T : IEntity
{
    protected EditableEntityViewModelBase(T entity, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(entity, entityCache, contexts)
    {
    }

    /// <summary>
    ///     Update the original Entity
    /// </summary>
    public abstract void UpdateEntity();

    /// <summary>
    ///     SaveAsync the original Entity to the database.
    /// </summary>
    public virtual async Task SaveAsync()
    {
        await using var ctx = await Contexts.CreateDbContextAsync();
        UpdateEntity();
        ctx.Update(Entity);
        await ctx.SaveChangesAsync();
    }
}