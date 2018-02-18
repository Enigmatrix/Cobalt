using Cobalt.Common.Data;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

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
