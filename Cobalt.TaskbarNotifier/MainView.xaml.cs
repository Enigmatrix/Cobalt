using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Input;

namespace Cobalt.TaskbarNotifier
{
    /// <summary>
    ///     Interaction logic for MainView.xaml
    /// </summary>
    public partial class MainView
    {
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

        private bool _set = false;

        private void SetPopupClose(object sender, RoutedEventArgs e)
        {
            if(_set) return;
            _set = true;
            Tray.TrayPopupResolved.Closed += (r, t) =>
            {
                (DataContext as MainViewModel).PopupClosed();
            };
        }

    }
}