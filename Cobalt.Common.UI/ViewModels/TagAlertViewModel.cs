using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagAlertViewModel : AlertViewModel
    {
        public TagAlertViewModel(TagAlert alert) : base(alert)
        {
            Tag = new TagViewModel(alert.Tag);
        }

        public TagViewModel Tag { get; set; }
    }
}