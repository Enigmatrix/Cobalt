using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Windows;
using System.Windows.Media;
using Caliburn.Micro;
using LiveCharts;
using LiveCharts.Wpf;
using LiveCharts.Wpf.Charts.Base;
using MahApps.Metro.Controls;

namespace Cobalt.Common.UI.Controls
{
    /// <summary>
    ///     Interaction logic for AppDurationLegend.xaml
    /// </summary>
    public partial class AppDurationLegend : IChartLegend
    {
        private List<SeriesViewModel> _series;
        private BindableCollection<SeriesReference> _chartSeries;

        public AppDurationLegend()
        {
            InitializeComponent();
            DataContext = this;
            Loaded += (o, e) =>
            {
                var chart = GetRootChart(this);
                ChartSeries = SyncedSeriesReferences(chart.Series);

                //not sure why, but this is called when 
                //the expansion of datacard is closed also

                //might lead to bugs? xD
            };
        }

        public List<SeriesViewModel> Series
        {
            get => _series;
            set
            {
                _series = value;
                OnPropertyChanged(nameof(Series));
            }
        }

        public BindableCollection<SeriesReference> ChartSeries
        {
            get => _chartSeries;
            set
            {
                _chartSeries = value;
                OnPropertyChanged(nameof(ChartSeries));
            }
        }

        public event PropertyChangedEventHandler PropertyChanged;

        private static Chart GetRootChart(UIElement element)
        {
            var parent = element;
            while (!(parent is Chart)) parent = (UIElement) parent.GetParentObject();
            return (Chart) parent;
        }

        protected virtual void OnPropertyChanged(string propertyName = null)
        {
            if (PropertyChanged != null)
                PropertyChanged.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        public BindableCollection<SeriesReference> SyncedSeriesReferences(SeriesCollection coll)
        {
            var obs = new BindableCollection<SeriesReference>(coll.Cast<Series>().Select(x => new SeriesReference(x)));
            coll.NoisyCollectionChanged += (_, __) => { obs.Clear(); obs.AddRange(coll.Cast<Series>().Select(x => new SeriesReference(x))); };
            return obs;
        }
    }
        public class SeriesReference : NotifyPropertyChanged
        {
            private readonly Series _series;

            public SeriesReference(Series series)
            {
                _series = series;
            }

            public string Title
            {
                get => _series.Title;
                set => _series.Title = value;
            }

            public Brush Fill
            {
                get => _series.Fill;
                set => _series.Fill = value;
            }

            public bool IsVisible
            {
                get => _series.Visibility == Visibility.Visible;
                set => _series.Visibility = value ? Visibility.Visible : Visibility.Collapsed;
            }
        }
}