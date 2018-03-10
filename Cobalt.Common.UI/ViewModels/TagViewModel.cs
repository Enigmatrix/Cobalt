using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagViewModel : EntityViewModel
    {
        private string _name;
        private ObservableCollection<AppViewModel> _taggedApps;

        public TagViewModel(Tag tag) : base(tag)
        {
            Name = tag.Name;
            TaggedApps = new ObservableCollection<AppViewModel>();
        }

        public string Name
        {
            get => _name;
            set => Set(ref _name, value);
        }

        public ObservableCollection<AppViewModel> TaggedApps
        {
            get => _taggedApps;
            set => Set(ref _taggedApps, value);
        }
    }
}
