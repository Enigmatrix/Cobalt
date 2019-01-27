using System;
using System.Reactive.Linq;
using System.Windows;
using Cobalt.Common.IoC;

namespace Cobalt
{
    /// <summary>
    ///     Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        public MainWindow()
        {
            InitializeComponent();
            Loaded += OnLoaded;
        }

        private void OnLoaded(object sender, RoutedEventArgs e)
        {
            var svc = IoCService.Instance.Resolve<Service>();
            svc.Switches()
                .ObserveOnDispatcher()
                .Subscribe(x => { Usages.Items.Add(x.Previous.App.Path); });
        }
    }
}