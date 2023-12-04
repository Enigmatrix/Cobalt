using System;
using System.Collections.Generic;
using Avalonia.Controls;
using Cobalt.Common.ViewModels;
using Cobalt.Views.Pages;
using FluentAvalonia.UI.Controls;
using FluentAvalonia.UI.Media.Animation;
using FluentAvalonia.UI.Navigation;
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
        private readonly Dictionary<string, Control> _pages = new()
        {
#if DEBUG
            [mvm.Experiments.Name] = new ExperimentsPage { ViewModel = mvm.Experiments },
#endif
            [mvm.Home.Name] = new HomePage { ViewModel = mvm.Home },
            [mvm.Apps.Name] = new AppsPage { ViewModel = mvm.Apps },
            [mvm.Tags.Name] = new TagsPage { ViewModel = mvm.Tags },
            [mvm.Alerts.Name] = new AlertsPage { ViewModel = mvm.Alerts },
            [mvm.History.Name] = new HistoryPage { ViewModel = mvm.History }
        };

        public Control GetPage(Type srcType)
        {
            throw new NotSupportedException();
        }

        public Control GetPageFromObject(object target)
        {
            return _pages[(string)target];
        }
    }
}