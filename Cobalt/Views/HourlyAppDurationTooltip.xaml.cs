using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Wpf;

namespace Cobalt.Views
{
    /// <summary>
    ///     Interaction logic for HourlyAppDurationTooltip.xaml
    /// </summary>
    /// //TODO INSTEAD OF USING THIS, USE THE APPDURATIONTOOLTIP
    public partial class HourlyAppDurationTooltip : IChartTooltip
    {
        private TooltipData _data;
        private string _selectedPath;
        private List<DataPointViewModel> _sortedPoints;

        public HourlyAppDurationTooltip()
        {
            InitializeComponent();
            DataContext = this;
        }

        public List<DataPointViewModel> SortedPoints
        {
            get =>
                _sortedPoints;
            set => Set(ref _sortedPoints, value);
        }

        public string SelectedPath
        {
            get => _selectedPath;
            set => Set(ref _selectedPath, value);
        }

        public event PropertyChangedEventHandler PropertyChanged;

        public TooltipData Data
        {
            get => _data;
            set
            {
                Set(ref _data, value);
                SortedPoints = _data.Points.Where(x => x.ChartPoint.Participation != 0.0)
                    .OrderByDescending(x => x.ChartPoint.Participation).ToList();
                SelectedPath = ((IAppDurationViewModel) _data.SenderSeries.ChartPoints.First().Instance).App.Path;
            }
        }

        public TooltipSelectionMode? SelectionMode { get; set; }

        protected virtual void Set<T>(ref T m, T val, [CallerMemberName] string propertyName = null)
        {
            m = val;
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}