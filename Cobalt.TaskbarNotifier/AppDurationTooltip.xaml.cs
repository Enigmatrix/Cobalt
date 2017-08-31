using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using LiveCharts;
using LiveCharts.Wpf;

namespace Cobalt.TaskbarNotifier
{
    /// <summary>
    /// Interaction logic for AppDurationTooltip.xaml
    /// </summary>
    public partial class AppDurationTooltip : IChartTooltip
    {
        private TooltipData _data;
        private List<DataPointViewModel> _sortedPoints;

        public AppDurationTooltip()
        {
            InitializeComponent();
            DataContext = this;
        }

        public event PropertyChangedEventHandler PropertyChanged;

        public TooltipData Data
        {
            get => _data;
            set
            {
                _data = value;
                OnPropertyChanged();
                SortedPoints = _data.Points.OrderByDescending(x => x.ChartPoint.Participation).ToList();
            }
        }

        public List<DataPointViewModel> SortedPoints
        {
            get => _sortedPoints;
            set { _sortedPoints = value; OnPropertyChanged(); }
        }

        public TooltipSelectionMode? SelectionMode { get; set; }

        protected virtual void OnPropertyChanged([CallerMemberName]string propertyName = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
