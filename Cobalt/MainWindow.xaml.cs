using System;
using System.Reactive.Disposables;
using System.Windows;
using Cobalt.Common.IoC;
using ReactiveUI;

namespace Cobalt
{
    /// <summary>
    ///     Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : ReactiveWindow<MainViewModel>
    {
        public MainWindow()
        {
            InitializeComponent();
            ViewModel = IoCService.Instance.Resolve<MainViewModel>();
            this.WhenActivated(regs =>
            {
                this.OneWayBind(ViewModel, vm => vm.AppUsages, v => v.Usages.ItemsSource)
                    .DisposeWith(regs);
            });
        }

        private void Throw(object sender, RoutedEventArgs e)
        {
            throw new NotImplementedException();
        }
    }
}