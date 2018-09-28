using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppViewModel : EntityViewModel<App>
    {

        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _appHourlyChunks;

        public AppViewModel(App app) : base(app)
        {
            Name = app.Name;
            Path = app.Path;
            Color = app.Color;
            Tags = app.Tags?.Select(t => new TagViewModel(t));
            Icon = app.Icon;
            IsDirty = false;
            this.PropertyChanges()
                .Where(x => x != nameof(IsDirty))
                .Subscribe(x => IsDirty = true)
                .ManageUsing(Resources);
        }

        public void Update()
        {
            Repository.UpdateApp(Entity);
            IsDirty = false;
        }

        public bool IsDirty { get; set; }

        public string Name
        {
            get => Entity.Name;
            set {
                if (value == Entity.Name) return;
                Entity.Name = value;
                NotifyOfPropertyChange();
            }
        }

        public string Path
        {
            get => Entity.Path;
            set {
                if (value == Entity.Path) return;
                Entity.Path = value;
                NotifyOfPropertyChange();
            }
        }

        public string Color
        {
            get => Entity.Color;
            set {
                if (value == Entity.Color) return;
                Entity.Color = value;
                NotifyOfPropertyChange();
            }
        }

        public IObservable<TagViewModel> Tags { get; }
        public IObservable<byte[]> Icon { get; }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> AppHourlyChunks =>
            _appHourlyChunks ?? (_appHourlyChunks = AppStats.GetChunkedAppDurations(TimeSpan.FromHours(1),
                d => d.Date.AddHours(d.Hour), DateTime.Today).Where(x => x.Value.App.Path == Path));
    }
}