using System;
using System.Reactive.Linq;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppViewModel : EntityViewModel
    {
        private string _color;
        private string _name;
        private string _path;

        public AppViewModel(App app) : base(app)
        {
            Name = app.Name;
            Path = app.Path;
            Color = app.Color;
            Tags = app.Tags?.Select(t => new TagViewModel(t));
            Icon = app.Icon;
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

        public string Color
        {
            get => _color;
            set => Set(ref _color, value);
        }

        public IObservable<TagViewModel> Tags { get; }
        public IObservable<byte[]> Icon { get; }
    }
}