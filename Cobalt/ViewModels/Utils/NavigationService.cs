using Caliburn.Micro;
using Cobalt.Common.UI;
using Cobalt.ViewModels.Pages;
using System.Collections.Generic;
using System.ComponentModel;

namespace Cobalt.ViewModels.Utils
{
    public interface INavigationService : INotifyPropertyChanged
    {
        void NavigateTo<T>(object arg=null) where T : PageViewModel;
        void NavigateToExisting<T>() where T : PageViewModel;

        PageViewModel Current { get; }
    }
    public class NavigationService : NotifyPropertyChanged, INavigationService
    {
        private PageViewModel _current;
        private List<PageViewModel> _existingPages = new List<PageViewModel>();



        public PageViewModel Current { get => _current; set => Set(ref _current, value); }

        public void NavigateTo<T>(object arg = null) where T : PageViewModel
        {
            if(Current == null)
            {

            }
        }

        public void NavigateToExisting<T>() where T : PageViewModel
        {
            throw new System.NotImplementedException();
        }
    }
}
