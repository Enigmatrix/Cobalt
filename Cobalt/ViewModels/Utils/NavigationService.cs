using System;
using System.Collections.Generic;
using System.ComponentModel;
using Caliburn.Micro;
using Cobalt.Common.IoC;
using Cobalt.Common.UI;
using Cobalt.ViewModels.Pages;
using Cobalt.Views.Pages;

namespace Cobalt.ViewModels.Utils
{
    public interface INavigationService : INotifyPropertyChanged, IDisposable
    {
        PageView ActivePage { get; }
        PageViewModel ActiveItem { get; }
        void NavigateTo<T>() where T : PageViewModel;
        void NavigateToType(Type value);
    }

    public class NavigationService : Conductor<PageViewModel>.Collection.OneActive, INavigationService
    {
        private readonly IResourceScope _resolver;
        private PageView _activePage;
        private readonly Dictionary<Type, (PageViewModel ViewModel, PageView View)> _existing;

        public NavigationService(IResourceScope resolver)
        {
            _resolver = resolver;
            _existing = new Dictionary<Type, (PageViewModel ViewModel, PageView View)>();
            //needed for conductors that aren't shown
            ((IActivate)this).Activate();
        }

        public PageView ActivePage
        {
            get => _activePage;
            set { _activePage = value ;NotifyOfPropertyChange();}
        }

        public void NavigateTo<T>() where T : PageViewModel
        {
            NavigateToType(typeof(T));
        }

        public void NavigateToType(Type value)
        {
            if (value == null || !typeof(PageViewModel).IsAssignableFrom(value)) return;
            if (_existing.ContainsKey(value))
            {
                var (vm, page) = _existing[value];
                ActivePage = page;
                ActivateItem(vm);
            }
            else
            {
                var vm = _resolver.Resolve<PageViewModel>(value);
                ActivePage = (PageView)ViewLocator.LocateForModel(vm, null, null);
                ViewModelBinder.Bind(vm, _activePage, null);
                ActivateItem(vm);
                _existing[value] = (vm, ActivePage);
            }
        }

        public void Dispose()
        {
            _resolver.Dispose();
        }
    }
}