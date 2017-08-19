using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public interface IAppDurationViewModel : IViewModel, IHasDuration
    {
        App App { get; set; }
    }

    public class AppDurationViewModel : ViewModelBase, IAppDurationViewModel
    {
        private App _app;
        private TimeSpan _duration;

        public AppDurationViewModel(App app) : this(app, TimeSpan.Zero)
        {
        }

        public AppDurationViewModel(App app, TimeSpan duration)
        {
            App = app;
            Duration = duration;
        }

        public App App
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