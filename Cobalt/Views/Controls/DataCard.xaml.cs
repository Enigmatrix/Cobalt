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

        public static readonly DependencyProperty ExpandedProperty =
            DependencyProperty.RegisterAttached(
                "Expanded",
                typeof(bool),
                typeof(DataCard),
                new PropertyMetadata(false)
            );

        public static void SetExpanded(UIElement element, bool value)
        {
            element.SetValue(ExpandedProperty, value);
        }

        public static bool GetExpanded(UIElement element)
        {
            return (bool) element.GetValue(ExpandedProperty);
        }


        private async void DataCardExpand(object sender, RoutedEventArgs e)
        {
            var container = (ContentControl) FindResource("DialogContainer");
            var root = (Grid) GetTemplateChild("Root");
            var contentHolder = (ContentPresenter) GetTemplateChild("ContentHolder");
            var parent = (ContentControl) root.Parent;
            var actualContent = (UIElement)contentHolder.Content;
            var chart = actualContent as Chart;

            if(chart != null)
                chart.Loaded += (_, __) => chart.Update(true,true);

            parent.Content = null;
            container.Content = root;

            await this.ShowDialog(container, (o, oe) =>
            {
                SetExpanded(actualContent, true);
                container.UpdateLayout();
                chart?.Update(true,true);
            }, (o, ce) =>
            {
                SetExpanded(actualContent, false);
                container.Content = null;
                parent.Content = root;
                parent.UpdateLayout();
                chart?.Update(true, true);
            });
        }
    }
}