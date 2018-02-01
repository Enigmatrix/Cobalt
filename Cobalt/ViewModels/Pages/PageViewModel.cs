using Cobalt.Common.IoC;
using Cobalt.Common.UI;

namespace Cobalt.ViewModels.Pages
{
    public abstract class PageViewModel : ViewModelBase
    {
        private IResourceScope _resources;

        protected PageViewModel(IResourceScope scope)
        {
            GlobalResources = scope;
        }

        public IResourceScope GlobalResources { get; set; }

        public IResourceScope Resources
        {
            get => _resources;
            set => Set(ref _resources, value);
        }


        protected override void OnActivate()
        {
            Resources = GlobalResources.Subscope();
            OnActivate(Resources);
        }

        protected virtual void OnActivate(IResourceScope resources)
        {
        }

        protected override void OnDeactivate(bool close)
        {
            OnDeactivate(close, Resources);
            Resources.Dispose();
            if (close) GlobalResources.Dispose();
        }

        protected virtual void OnDeactivate(bool close, IResourceScope resources)
        {
        }
    }
}