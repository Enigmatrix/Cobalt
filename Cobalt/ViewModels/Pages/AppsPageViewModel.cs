using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;

namespace Cobalt.ViewModels.Pages
{
    public class AppsPageViewModel : PageViewModel
    {
        private string _appFilter;
        private IObservable<AppViewModel> _apps;

        public AppsPageViewModel(IResourceScope scope) : base(scope)
        {
        }
        
        public IObservable<AppViewModel> Apps
        {
            get => _apps;
            set => Set(ref _apps, value);
        }

        public string AppFilter
        {
            get => _appFilter;
            set => Set(ref _appFilter, value);
        }

        protected override void OnActivate(IResourceScope resources)
        {
            var repo = resources.Resolve<IEntityStreamService>();
            this.PropertyChanges(nameof(AppFilter))
                .Select(_ =>
                {
                    return repo.GetApps().Select(x => new AppViewModel(x))
                        .Where(x => x.Name != null)
                        .Where(x => x.Name.StrContains(AppFilter))
                        .ObserveOnDispatcher();
                })
                .Subscribe(apps => Apps = apps)
                .ManageUsing(resources);
        }

        protected override void OnDeactivate(bool close, IResourceScope resources)
        {
            Apps = null;
        }
    }
}