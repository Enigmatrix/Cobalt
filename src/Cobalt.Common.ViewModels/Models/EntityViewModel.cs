using Cobalt.Common.Data;
using Cobalt.Common.Data.Models;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Models;

public abstract class EntityViewModel<T> : ObservableValidator where T: Entity
{
    public T Inner { get; set; } = default!;

    protected EntityViewModelCache Cache { get; }

    public abstract void InitializeEntity(T entity);

    public long Id => Inner.Id;

    protected EntityViewModel(EntityViewModelCache cache)
    {
        Cache = cache;
    }
}

public abstract class EditableEntityViewModel<T> : EntityViewModel<T> where T: Entity
{
    protected readonly CobaltContext Context;

    protected EditableEntityViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(cache)
    {
        Context = ctx;
    }

    public abstract void SaveChanges();
}
