using System;
using Avalonia.Controls;
using Cobalt.Common.ViewModels;
using FluentAvalonia.UI.Controls;
using FluentAvalonia.UI.Media.Animation;
using FluentAvalonia.UI.Navigation;
using Microsoft.Extensions.DependencyInjection;
using ReactiveUI;

namespace Cobalt.Views;

public partial class MainWindow : Window, IViewFor<MainViewModel>
{
    public MainWindow()
    {
        InitializeComponent();
        // This WhenActivated block calls ViewModel's WhenActivated
        // block if the ViewModel implements IActivatableViewModel.
        this.WhenActivated(disposables =>
        {
            Frame.NavigationPageFactory ??= new NavFactory(ViewModel!);
#if DEBUG
            var startPage = ViewModel!.Experiments.Name;
#else
            var startPage = ViewModel!.Home.Name;
#endif
            Frame.NavigateFromObject(startPage, new FrameNavigationOptions
            {
                IsNavigationStackEnabled = false,
                TransitionInfoOverride = new EntranceNavigationTransitionInfo()
            });
        });
    }

    object? IViewFor.ViewModel
    {
        get => DataContext;
        set => DataContext = value;
    }

    public MainViewModel? ViewModel
    {
        get => DataContext as MainViewModel ??
               throw new InvalidOperationException($"DataContext is not {nameof(MainViewModel)}");
        set => DataContext = value;
    }

    private void NavigateTo(object? sender, NavigationViewItemInvokedEventArgs e)
    {
        Frame.NavigateFromObject(e.InvokedItem, new FrameNavigationOptions
        {
            IsNavigationStackEnabled = false,
            TransitionInfoOverride = new SlideNavigationTransitionInfo()
        });
    }

    public record NavFactory(MainViewModel mvm) : INavigationPageFactory
    {
        public Control GetPage(Type srcType)
        {
            var vm = (ViewModelBase)mvm.Provider.GetRequiredService(srcType);
            return GetViewFromViewModel(vm);
        }

        public Control GetPageFromObject(object target)
        {
            var vm = mvm.Pages[(string)target];
            return GetViewFromViewModel(vm);
        }

        public Control GetViewFromViewModel(ViewModelBase vm)
        {
            var vType = typeof(IViewFor<>).MakeGenericType(vm.GetType());
            var v = (IViewFor)mvm.Provider.GetRequiredService(vType);
            v.ViewModel ??= vm;
            return (Control)v;
        }
    }
}