﻿using System.Collections.Generic;
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

        public AppDurationLegend()
        {
            InitializeComponent();
            Loaded += (o, e) =>
            {
                Chart = GetRootChart(this);
                DataContext = this;
                ChartSeries = SyncedSeriesReferences(Chart.Series);

                //not sure why, but this is called when 
                //the expansion of datacard is closed also

                //might lead to bugs? xD
            };
        }

        public Chart Chart { get; set; }

        public BindableCollection<SeriesReference> ChartSeries { get;set; }

        public List<SeriesViewModel> Series { get; set; }

        public event PropertyChangedEventHandler PropertyChanged;

        private static Chart GetRootChart(UIElement element)
        {
            var parent = element;
            while (!(parent is Chart)) parent = (UIElement) parent.GetParentObject();
            return (Chart) parent;
        }

        public BindableCollection<SeriesReference> SyncedSeriesReferences(SeriesCollection coll)
        {
            var obs = new BindableCollection<SeriesReference>(coll.Cast<Series>().Select(x => new SeriesReference(x)));
            coll.NoisyCollectionChanged += (_, __) =>
            {
                obs.Clear();
                obs.AddRange(coll.Cast<Series>().Select(x => new SeriesReference(x)));
            };
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