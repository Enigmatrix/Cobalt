using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Wpf;

namespace Cobalt.Common.UI.Controls
{
    /// <summary>
    ///     Interaction logic for AppDurationTooltip.xaml
    /// </summary>
    public partial class TagDurationTooltip : IChartTooltip
    {
        private TooltipData _data;

        public TagDurationTooltip()
        {
            InitializeComponent();
            DataContext = this;
        }

        public List<DataPointViewModel> SortedPoints { get; set; }

        public string SelectedPath { get; set; }

        public event PropertyChangedEventHandler PropertyChanged;

        public TooltipData Data
        {
            get => _data;
            set
            {
                Set(ref _data, value);
                SortedPoints = _data.Points.Where(x => x.ChartPoint.Participation != 0.0)
                    .OrderByDescending(x => x.ChartPoint.Participation).ToList();
                SelectedPath = ((TagDurationViewModel) _data.SenderSeries.ChartPoints.First().Instance).Tag.Name;
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