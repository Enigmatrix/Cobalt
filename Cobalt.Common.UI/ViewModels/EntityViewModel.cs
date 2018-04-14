using Cobalt.Common.Analysis;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;

namespace Cobalt.Common.UI.ViewModels
{
    public class EntityViewModel : ViewModelBase
    {
        private int _id;

        public EntityViewModel(Entity entity)
        {
            Entity = entity;
        }

        public int Id
        {
            get => _id;
            set => Set(ref _id, value);
        }

        private IResourceScope _resources;
        private IDbRepository _repository;
        private IAppStatsStreamService _appStats;

        protected IResourceScope Resources => _resources ?? (_resources = IoCService.Instance.Resolve<IResourceScope>().Subscope());

        protected IDbRepository Repository => _repository ?? (_repository = Resources.Resolve<IDbRepository>());
        
        protected IAppStatsStreamService AppStats => _appStats ?? (_appStats = Resources.Resolve<IAppStatsStreamService>());

        public Entity Entity { get; }

        public override void Dispose()
        {
            base.Dispose();
            Resources.Dispose();
        }
    }
}