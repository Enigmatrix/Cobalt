using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Wpf;

namespace Cobalt.Views
{
    /// <summary>
    /// Interaction logic for HourlyAppDurationTooltip.xaml
    /// </summary>
    /// //TODO INSTEAD OF USING THIS, USE THE APPDURATIONTOOLTIP
    public partial class HourlyAppDurationTooltip : IChartTooltip
    {
        private List<DataPointViewModel> _sortedPoints;
        private TooltipData _data;
        private string _selectedPath;

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

        public event PropertyChangedEventHandler PropertyChanged;

        public TooltipData Data
        {
            get => _data;
            set
            {
                Set(ref _data, value);
                SortedPoints = _data.Points.Where(x => x.ChartPoint.Participation != 0.0).OrderByDescending(x => x.ChartPoint.Participation).ToList();
                SelectedPath = ((IAppDurationViewModel) _data.SenderSeries.ChartPoints.First().Instance).App.Path;
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
