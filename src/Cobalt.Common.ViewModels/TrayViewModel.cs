using System.Collections.ObjectModel;
using Cobalt.Common.ViewModels.Entities;

namespace Cobalt.Common.ViewModels
{
    public class TrayViewModel : ViewModelBase
    {
        public ReadOnlyObservableCollection<AlertViewModel> Alerts { get; }
        public ReadOnlyObservableCollection<AppDurationViewModel> AppDurations { get; }

        public TrayViewModel()
        {
            
        }
    }
}