using System;
using System.Runtime.Serialization.Formatters;
using Cobalt.Common.UI;
using Cobalt.ViewModels.Utils;

namespace Cobalt.ViewModels
{
    public class MainViewModel : ViewModelBase
    {

        public MainViewModel(INavigationService navSvc)
        {
            NavigationService = navSvc;
        }

        public Type ActiveItem
        {
            set => NavigationService.NavigateToType(value);
        }

        public INavigationService NavigationService { get; }
    }
}