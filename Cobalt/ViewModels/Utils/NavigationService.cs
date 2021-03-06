﻿using System;
using System.ComponentModel;
using System.Threading.Tasks;
using Caliburn.Micro;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;
using Cobalt.ViewModels.Pages;
using Cobalt.Views.Dialogs;
using Cobalt.Views.Pages;
using MaterialDesignThemes.Wpf;

namespace Cobalt.ViewModels.Utils
{
    public interface INavigationService : INotifyPropertyChanged, IDisposable
    {
        PageView ActivePage { get; }
        PageViewModel ActiveItem { get; }
        void NavigateTo<T>() where T : PageViewModel;
        void NavigateToType(Type value);
        Task<object> ShowDialog<T>(params object[] args) where T : Dialog, new();
    }

    public class NavigationService : Conductor<PageViewModel>.Collection.OneActive, INavigationService
    {
        private readonly Cache<Type, PageViewModel> _existingViewModels;
        private readonly Cache<Type, PageView> _existingViews;
        private readonly IResourceScope _resolver;

        public NavigationService(IResourceScope resolver)
        {
            _resolver = resolver;
            _existingViewModels = new Cache<Type, PageViewModel>();
            _existingViews = new Cache<Type, PageView>();
            //needed for conductors that aren't shown
            ((IActivate) this).Activate();
        }

        public PageView ActivePage { get; set; }

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
                ViewModelBinder.Bind(vm, ActivePage, null);
                ActivateItem(vm);

                _existingViewModels.Add(value, vm);
                _existingViews.Add(value, ActivePage);
            }
        }

        public void Dispose()
        {
            _resolver.Dispose();
        }

        public async Task<object> ShowDialog<T>(params object[] args) where T : Dialog, new()
        {
            var dialog = new T();
            dialog.Prepare(args);
            return await ActivePage.ShowDialog(dialog);
        }
    }
}