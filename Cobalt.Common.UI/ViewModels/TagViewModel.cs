using Caliburn.Micro;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagViewModel : EntityViewModel
    {
        private string _name;
        private BindableCollection<AppViewModel> _taggedApps;

        public TagViewModel(Tag tag) : base(tag)
        {
            Name = tag.Name;
        }

        public string Name
        {
            get => _name;
            set => Set(ref _name, value);
        }

        public BindableCollection<AppViewModel> TaggedApps => _taggedApps ?? (_taggedApps = new BindableCollection<AppViewModel>());
    }
}