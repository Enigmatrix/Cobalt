using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppDurationViewModel : EntityViewModel, IHasDuration
    {
        public AppDurationViewModel(App app) : this(app, TimeSpan.Zero)
        {
        }

        public AppDurationViewModel(App app, TimeSpan duration) : base(app)
        {
            App = new AppViewModel(app);
            Duration = duration;
        }

        public AppViewModel App { get;set; }

        public TimeSpan Duration { get;set; }
    }
}