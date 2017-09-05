using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Wpf;

namespace Cobalt.TaskbarNotifier
{
    /// <summary>
    ///     Interaction logic for AppDurationTooltip.xaml
    /// </summary>
    public partial class AppDurationTooltip : IChartTooltip
    {
        private TooltipData _data;
        private List<DataPointViewModel> _sortedPoints;
        private string _selectedPath;

        public AppDurationTooltip()
        {
            InitializeComponent();
            DataContext = this;
        }

        public List<DataPointViewModel> SortedPoints
        {
            get => _sortedPoints;
            set => Set(ref _sortedPoints, value);
        }

        public event PropertyChangedEventHandler PropertyChanged;

        public TooltipData Data
        {
            get => _data;
            set
            {
                Set(ref _data, value);
                SortedPoints = _data.Points.OrderByDescending(x => x.ChartPoint.Participation).ToList();
                SelectedPath = ((IAppDurationViewModel) _data.SenderSeries.ChartPoints.Single().Instance).App.Path;
            }
        }

        public string SelectedPath
        {
            get => _selectedPath;
            set => Set(ref _selectedPath, value);
        }

        public TooltipSelectionMode? SelectionMode { get; set; }

        protected virtual void Set<T>(ref T m, T val, [CallerMemberName] string propertyName = null)
        {
            m = val;
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}