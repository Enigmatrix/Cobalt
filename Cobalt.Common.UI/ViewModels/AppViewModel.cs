using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppViewModel : EntityViewModel
    {
        private string _color;
        private string _name;
        private string _path;
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _appHourlyChunks;

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

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> AppHourlyChunks =>
            _appHourlyChunks ?? (_appHourlyChunks = AppStats.GetChunkedAppDurations(TimeSpan.FromHours(1),
                d => d.Date.AddHours(d.Hour), DateTime.Today).Where(x => x.Value.App.Path == Path));
    }
}