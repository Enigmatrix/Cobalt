using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagViewModel : EntityViewModel
    {
        private string _name;

        public TagViewModel(Tag tag) : base(tag)
        {
            Name = tag.Name;
        }

        public string Name
        {
            get => _name;
            set => Set(ref _name, value);
        }
    }
}
