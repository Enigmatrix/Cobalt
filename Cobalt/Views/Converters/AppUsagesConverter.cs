using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Converters;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Views.Converters
{
    public class AppUsagesConverter : ObservableConverter<AppUsageViewModel, ObservableCollection<AppUsageViewModel>>
    {
        protected override ObservableCollection<AppUsageViewModel> Convert(IObservable<AppUsageViewModel> values, object parameter, IResourceScope manager)
        {
            var coll = new ObservableCollection<AppUsageViewModel>();
            values//.Buffer(TimeSpan.FromMilliseconds(200))
                .ObserveOnDispatcher().Subscribe(x => coll.Add(x)).ManageUsing(manager);
            return coll;
        }
    }
}
