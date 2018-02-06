using System.Windows;

namespace Cobalt.TaskbarNotifier
{
    /// <summary>
    ///     Interaction logic for MainView.xaml
    /// </summary>
    public partial class MainView
    {
        private bool _set;

        public MainView()
        {
            InitializeComponent();
        }

        private void TrayKeepToggle(object sender, RoutedEventArgs e)
        {
            if (Tray.TrayPopupResolved.StaysOpen)
            {
                Tray.TrayPopupResolved.StaysOpen = false;
                Tray.TrayPopupResolved.IsOpen = false;
            }
            else
            {
                Tray.TrayPopupResolved.StaysOpen = true;
            }
        }

        private void SetPopupClose(object sender, RoutedEventArgs e)
        {
            if (_set) return;
            _set = true;
            Tray.TrayPopupResolved.Closed += (r, t) => ((MainViewModel) DataContext).PopupClosed();
        }
    }
}