using System;
using System.Runtime.Serialization.Formatters;
using Cobalt.Common.UI;
using Cobalt.ViewModels.Pages;
using Cobalt.ViewModels.Utils;
using Cobalt.Views.Pages;

namespace Cobalt.ViewModels
{
    public class MainViewModel : ViewModelBase
    {

        public MainViewModel(INavigationService navSvc)
        {
            NavigationService = navSvc;
            NavigationService.NavigateTo<HomePageViewModel>();
        }

        public Type ActiveItem
        {
            get => NavigationService.ActiveItem.GetType();
            set { NavigationService.NavigateToType(value); NotifyOfPropertyChange();}
        }

        public INavigationService NavigationService { get; }
    }
}