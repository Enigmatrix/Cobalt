using System;
using System.Collections.ObjectModel;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Converters;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Views.Converters
{
    public class TagsConverter : ObservableConverter<TagViewModel, ObservableCollection<TagViewModel>>
    {
        protected override ObservableCollection<TagViewModel> Convert(IObservable<TagViewModel> values,
            object parameter, IResourceScope manager)
        {
            var coll = new ObservableCollection<TagViewModel>();
            values.Subscribe(x => coll.Add(x)).ManageUsing(manager);
            return coll;
        }
    }
}