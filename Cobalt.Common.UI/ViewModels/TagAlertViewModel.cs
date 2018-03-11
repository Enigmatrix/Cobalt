using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagAlertViewModel : AlertViewModel
    {
        private TagViewModel _tag;

        public TagAlertViewModel(TagAlert alert) : base(alert)
        {
            Tag = new TagViewModel(alert.Tag);
        }

        public TagViewModel Tag
        {
            get => _tag;
            set => Set(ref _tag, value);
        }
    }
}