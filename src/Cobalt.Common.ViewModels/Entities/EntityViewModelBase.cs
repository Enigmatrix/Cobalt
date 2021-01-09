namespace Cobalt.Common.ViewModels.Entities
{
    public abstract class EntityViewModelBase<T> : ViewModelBase
    {
        protected readonly IEntityManager _mgr;

        protected EntityViewModelBase(IEntityManager mgr)
        {
            _mgr = mgr;
        }

        public long Id { get; protected set; }
    }
}