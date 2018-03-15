using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;
using Cobalt.ViewModels.Extended;

namespace Cobalt.ViewModels.Pages
{
    public class AppsPageViewModel : PageViewModel
    {
        private string _appFilter;
        private IObservable<ExtendedAppViewModel> _apps;

        public AppsPageViewModel(IResourceScope scope) : base(scope)
        {
        }

        public IObservable<ExtendedAppViewModel> Apps
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
                    return repo.GetApps().Select(x => new ExtendedAppViewModel(x, resources))
                        .Where(x => x.Name != null)
                        .Where(x => x.Name.StrContains(AppFilter))
                        .ObserveOnDispatcher();
                })
                .Subscribe(apps => Apps = apps)
                .ManageUsing(resources);
        }
    }
}