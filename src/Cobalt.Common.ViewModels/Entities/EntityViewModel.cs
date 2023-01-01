using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public abstract class EntityViewModel<T> : ObservableValidator where T : Entity
{
    protected EntityViewModel(EntityViewModelCache cache)
    {
        Cache = cache;
    }

    public T Inner { get; set; } = default!;

    protected EntityViewModelCache Cache { get; }

    public long Id => Inner.Id;

    public abstract void InitializeEntity(T entity);
}

public abstract class EditableEntityViewModel<T> : EntityViewModel<T> where T : Entity
{
    protected readonly CobaltContext Context;

    protected EditableEntityViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(cache)
    {
        Context = ctx;
    }

    public abstract void SaveChanges();
}