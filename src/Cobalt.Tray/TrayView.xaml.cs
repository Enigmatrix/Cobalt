using System.Reactive.Disposables;
using ReactiveUI;

namespace Cobalt.Tray
{
    /// <summary>
    ///     Interaction logic for TrayView.xaml
    /// </summary>
    public partial class TrayView
    {
        public TrayView()
        {
            InitializeComponent();

            this.WhenActivated(regs =>
            {
                this.OneWayBind(ViewModel, 
                    vm => vm.AppDurations, 
                    v => v.Data.ItemsSource)
                    .DisposeWith(regs);

                this.Data.ListenPropertyChange = true;
            });
        }
    }
}