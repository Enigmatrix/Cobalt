using System;
using System.Data;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.TaskbarNotifier
{
    public class MainViewModel : ViewModelBase
    {
        private BindableCollection<AppDurationViewModel> _appDurations
            = new BindableCollection<AppDurationViewModel>();

        public MainViewModel(IResourceScope res)
        {
            Global = res;
        }

        public IResourceScope Global { get; set; }
        public IResourceScope Current { get; set; }

        public BindableCollection<AppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }

        public void PopupOpened()
        {
            Current = Global.Subscope();
            var tats = Current.Resolve<IDbConnection>();
            var stats = Current.Resolve<IAppStatsStreamService>();

            /*
             * var globalClock = resolve(clock)
             * 
             * when app.Duration += d,
             * check if d == Timespan.Zero, if it is then
             * add clock.tick to appdur
             * 
             * clock.tick increments current appdur every <period>
             */

            stats.GetAppDurations(DateTime.Today)
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d => appDur.Duration += d)
                        .ManageUsing(Current);

                    return appDur;
                })
                .Subscribe(x => AppDurations.Add(x))
                .ManageUsing(Current);
        }

        public void PopupClosed()
        {
            AppDurations.Clear();
            Current.Dispose();
        }
    }
}