using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;

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

    public abstract class EntityViewModelBase<T> : EntityViewModelBase where T : IEntity
    {
        protected EntityViewModelBase(T entity, IEntityManager manager) : base(manager)
        {
            Id = entity.Id;
        }

        public override int GetHashCode()
        {
            return Id.GetHashCode();
        }

        public override bool Equals(object? obj)
        {
            return (obj as EntityViewModelBase<T>)?.Id == Id;
        }
    }

    public abstract class MutableEntityViewModelBase<T> : EntityViewModelBase<T> where T : IEntity
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