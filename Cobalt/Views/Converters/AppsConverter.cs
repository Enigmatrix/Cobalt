using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Converters;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Views.Converters
{
    public class AppsConverter : ObservableConverter<AppViewModel, ObservableCollection<AppViewModel>>
    {
        protected override ObservableCollection<AppViewModel> Convert(IObservable<AppViewModel> values, object parameter, IResourceScope manager)
        {
            var coll = new ObservableCollection<AppViewModel>();
            values.Subscribe(x => coll.Add(x)).ManageUsing(manager);
            return coll;
        }
    }
}
