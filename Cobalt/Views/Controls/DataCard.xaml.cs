using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using LiveCharts;
using LiveCharts.Wpf;
using LiveCharts.Wpf.Charts.Base;
using MaterialDesignThemes.Wpf;

namespace Cobalt.Views.Controls
{
    /// <summary>
    ///     Interaction logic for DataCard.xaml
    /// </summary>
    public partial class DataCard
    {
        public DataCard()
        {
            InitializeComponent();
        }

        private async void DataCardExpand(object sender, RoutedEventArgs e)
        {
            var container = (ContentControl) FindResource("DialogContainer");
            var root = (Grid) GetTemplateChild("Root");
            var contentHolder = (ContentPresenter) GetTemplateChild("ContentHolder");
            var parent = (ContentControl) root.Parent;

            parent.Content = null;
            container.Content = root;

            await this.ShowDialog(container, (o, oe) =>
            {
                container.UpdateLayout();
                if (contentHolder.Content is Chart chart)
                {
                    chart.Update(true, true);

                }
            }, (o, ce) =>
            {
                container.Content = null;
                parent.Content = root;
                root.UpdateLayout();
                if (contentHolder.Content is Chart chart)
                    chart.Update(true, true);
            });
        }
    }
}