using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Pages
{
    public class AppsPageViewModel : PageViewModel
    {
        private IObservable<AppViewModel> _apps;

        public AppsPageViewModel(IResourceScope scope) : base(scope)
        {
        }

        public IObservable<AppViewModel> Apps
        {
            get => _apps;
            set => Set(ref _apps, value);
        }

        protected override void OnActivate(IResourceScope resources)
        {
            var repo = resources.Resolve<IEntityStreamService>();
            Apps = repo.GetApps().Select(x => new AppViewModel(x));
        }
    }
}