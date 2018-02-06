using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppViewModel : EntityViewModel
    {
        private string _name;
        private string _path;

        public AppViewModel(App app) : base(app)
        {
            Name = app.Name;
            Path = app.Path;
            Tags = app.Tags?.Select(t => new TagViewModel(t));
        }

        public string Name
        {
            get => _name;
            set => Set(ref _name, value);
        }

        public string Path
        {
            get => _path;
            set => Set(ref _path, value);
        }

        public IObservable<TagViewModel> Tags { get; }
    }
}
