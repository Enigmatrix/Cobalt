using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppViewModel : EntityViewModel
    {
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _appHourlyChunks;

        public AppViewModel(App app) : base(app)
        {
            Name = app.Name;
            Path = app.Path;
            Color = app.Color;
            Tags = app.Tags?.Select(t => new TagViewModel(t));
            Icon = app.Icon;
        }

        public string Name { get; set; }

        public string Path { get; set; }

        public string Color { get; set; }

        public IObservable<TagViewModel> Tags { get; }
        public IObservable<byte[]> Icon { get; }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> AppHourlyChunks =>
            _appHourlyChunks ?? (_appHourlyChunks = AppStats.GetChunkedAppDurations(TimeSpan.FromHours(1),
                d => d.Date.AddHours(d.Hour), DateTime.Today).Where(x => x.Value.App.Path == Path));
    }
}