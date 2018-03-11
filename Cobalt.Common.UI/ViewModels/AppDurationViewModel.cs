using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppDurationViewModel : EntityViewModel, IHasDuration
    {
        private AppViewModel _app;
        private TimeSpan _duration;

        public AppDurationViewModel(App app) : this(app, TimeSpan.Zero)
        {
        }

        public AppDurationViewModel(App app, TimeSpan duration) : base(app)
        {
            App = new AppViewModel(app);
            Duration = duration;
        }

        public AppViewModel App
        {
            get => _app;
            set => Set(ref _app, value);
        }

        public TimeSpan Duration
        {
            get => _duration;
            set => Set(ref _duration, value);
        }
    }
}