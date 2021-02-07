using System.Windows;
using Cobalt.Common.ViewModels;
using Splat;

namespace Cobalt.Tray
{
    /// <summary>
    ///     Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        public MainWindow()
        {
            InitializeComponent();
        }

        public static TrayViewModel InnerViewModel =>
            Locator.Current.GetService<TrayViewModel>();
    }
}