using System;
using System.ComponentModel;
using Caliburn.Micro;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;
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
        private readonly Cache<Type, PageViewModel> _existingViewModels;
        private readonly Cache<Type, PageView> _existingViews;
        private readonly IResourceScope _resolver;
        private PageView _activePage;

        public NavigationService(IResourceScope resolver)
        {
            _resolver = resolver;
            _existingViewModels = new Cache<Type, PageViewModel>();
            _existingViews = new Cache<Type, PageView>();
            //needed for conductors that aren't shown
            ((IActivate) this).Activate();
        }

        public PageView ActivePage
        {
            get => _activePage;
            set
            {
                _activePage = value;
                NotifyOfPropertyChange();
            }
        }

        public void NavigateTo<T>() where T : PageViewModel
        {
            NavigateToType(typeof(T));
        }

        public void NavigateToType(Type value)
        {
            if (value == null || !typeof(PageViewModel).IsAssignableFrom(value)) return;
            if (_existingViewModels.TryGetValue(value, out var viewModel) &&
                _existingViews.TryGetValue(value, out var view))
            {
                ActivePage = view;
                ActivateItem(viewModel);
            }
            else
            {
                var vm = _resolver.Resolve<PageViewModel>(value);
                ActivePage = (PageView) ViewLocator.LocateForModel(vm, null, null);
                ViewModelBinder.Bind(vm, _activePage, null);
                ActivateItem(vm);

                _existingViewModels.Add(value, vm);
                _existingViews.Add(value, ActivePage);
            }
        }

        public void Dispose()
        {
            _resolver.Dispose();
        }
    }
}