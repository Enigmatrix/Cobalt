using System.Reactive.Disposables;
using ReactiveUI;

namespace Cobalt.Common.UI.ViewModels
{
    public abstract class ViewModelBase : ReactiveObject, ISupportsActivation
    {
        public ViewModelActivator Activator { get; }

        public ViewModelBase()
        {
            Activator = new ViewModelActivator();
            this.WhenActivated(() =>
            {
                var disp = new CompositeDisposable();
                Activate(disp);
                return disp;
            });
        }

        protected virtual void Activate(CompositeDisposable disp)
        {

        }
    }
}
