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
        PageViewModel CurrentViewModel { get; }
        PageView CurrentView { get; }
        void NavigateTo<T>() where T : PageViewModel;
        void NavigateToType(Type value);
    }

    public class NavigationService : NotifyPropertyChanged, INavigationService
    {
        private readonly IResourceScope _resolver;
        private PageView _currentView;
        private PageViewModel _currentViewModel;
        private readonly Dictionary<Type, (PageViewModel ViewModel, PageView View)> _existing;

        public NavigationService(IResourceScope resolver)
        {
            _resolver = resolver;
            _existing = new Dictionary<Type, (PageViewModel ViewModel, PageView View)>();
        }

        public PageViewModel CurrentViewModel
        {
            get => _currentViewModel;
            set => Set(ref _currentViewModel, value);
        }

        public PageView CurrentView
        {
            get => _currentView;
            set => Set(ref _currentView, value);
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
                var (viewModel, view) = _existing[value];
                CurrentViewModel = viewModel;
                CurrentView = view;
            }
            else
            {
                CurrentViewModel = _resolver.Resolve<PageViewModel>(value);
                CurrentView = (PageView)ViewLocator.LocateForModel(CurrentViewModel, null, null);
                ViewModelBinder.Bind(CurrentViewModel, CurrentView, null);
                _existing[value] = (CurrentViewModel, CurrentView);
            }
        }

        public void Dispose()
        {
            _resolver.Dispose();
        }
    }
}