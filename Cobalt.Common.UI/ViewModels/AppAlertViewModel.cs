using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppAlertViewModel : AlertViewModel
    {
        private AppViewModel _app;

        public AppAlertViewModel(AppAlert alert) : base(alert)
        {
            App = new AppViewModel(alert.App);
        }

        public AppViewModel App
        {
            get => _app;
            set => Set(ref _app, value);
        }
    }
}