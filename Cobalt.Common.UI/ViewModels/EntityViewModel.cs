using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.UI.ViewModels
{
    public abstract class EntityViewModel : ViewModelBase
    {
        public Entity Entity { get; }

        protected EntityViewModel(Entity entity)
        {
            Entity = entity;
        }

        public long Id => Entity.Id;
    }

    public abstract class EntityViewModel<T> : EntityViewModel
        where T : Entity
    {
        protected EntityViewModel(T entity) : base(entity)
        {
        }

        public new T Entity => (T) base.Entity;
    }
}
