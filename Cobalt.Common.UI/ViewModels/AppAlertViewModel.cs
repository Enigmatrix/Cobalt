using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppAlertViewModel : AlertViewModel
    {
        public AppAlertViewModel(AppAlert alert) : base(alert)
        {
            App = new AppViewModel(alert.App);
        }

        public AppViewModel App { get; set; }
    }
}