using System;
using System.Data;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.TaskbarNotifier
{
    public interface IMainViewModel : IViewModel
    {
        BindableCollection<AppDurationViewModel> AppDurations { get; }
    }

    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private BindableCollection<AppDurationViewModel> _appDurations
            = new BindableCollection<AppDurationViewModel>();

        public MainViewModel(IResourceScope res)
        {
            Global = res;
        }

        private IResourceScope Global { get; set; }
        private IResourceScope Current { get; set; }

        public BindableCollection<AppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }

        public void PopupOpened()
        {
            Current = Global.Subscope();
            var stats = Current.Resolve<IAppStatsStreamService>();
            var incrementor = Current.Resolve<IDurationIncrementor>();

            /*
             * var globalClock = resolve(clock)
             * 
             * when app.Duration += d,
             * check if d == Timespan.Zero, if it is then
             * add clock.tick to appdur
             * 
             * clock.tick increments current appdur every <period>
             * when diff appdur has d==timespan.zero, remove tick from appDur, and add prev thingy
             */

            stats.GetAppDurations(DateTime.Today)
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d =>
                        {
                            if (d is null)
                            {
                                //handle new app started here
                                incrementor.Increment(appDur);
                                appDur.Duration += TimeSpan.Zero;
                            }
                            else
                            {
                                incrementor.Release();
                                appDur.Duration += d.Value;
                            }
                        })
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