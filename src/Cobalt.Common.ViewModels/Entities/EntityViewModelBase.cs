using Cobalt.Common.Data;

namespace Cobalt.Common.ViewModels.Entities
{
    public abstract class EntityViewModelBase : ViewModelBase
    {
        protected readonly IEntityManager Manager;

        protected EntityViewModelBase(IEntityManager manager)
        {
            Manager = manager;
        }

        public long Id { get; protected set; }
    }

    public abstract class EntityViewModelBase<T> : EntityViewModelBase
    {
        protected EntityViewModelBase(T entity, IEntityManager manager) : base(manager)
        {
        }
    }

    public abstract class MutableEntityViewModelBase<T> : EntityViewModelBase<T>
    {
        protected readonly IDatabase Database;

        protected MutableEntityViewModelBase(T entity, IEntityManager manager, IDatabase db) : base(entity, manager)
        {
            // take reference to Database to save state etc
            Database = db;
        }

        public abstract void UpdateFromEntity(T entity);

        public abstract void Save();
    }
}