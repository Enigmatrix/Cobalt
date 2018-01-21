using System.Runtime.Serialization.Formatters;
using Cobalt.Common.UI;
using Cobalt.ViewModels.Utils;

namespace Cobalt.ViewModels
{
    public class MainViewModel : ViewModelBase
    {
        private object _activeItem;

        public MainViewModel(INavigationService navSvc)
        {
        }

        public object ActiveItem
        {
            get => _activeItem;
            set {
                if(value != null)
                    Set(ref _activeItem, value);
            }
        }
    }
}