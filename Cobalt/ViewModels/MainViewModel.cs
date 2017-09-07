using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Analysis;
using Cobalt.Common.UI;

namespace Cobalt.ViewModels
{
    public interface IMainViewModel : IViewModel
    {
        
    }
    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private IAppStatsStreamService Stats { get; }

        public MainViewModel(IAppStatsStreamService stats)
        {
            Stats = stats;
        }

        protected override void OnActivate()
        {
            Stats.GetAppUsages(DateTime.Today)
                .Subscribe();
        }
    }
}
