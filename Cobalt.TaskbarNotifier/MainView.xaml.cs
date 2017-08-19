using System.Windows;

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
            Tray.TrayPopupOpen += (_, e) => { Tray.TrayPopupResolved.StaysOpen = true; };
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
    }
}