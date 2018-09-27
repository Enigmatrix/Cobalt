using System;
using System.Diagnostics;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.TaskbarNotifier
{
    public class MainViewModel : ViewModelBase
    {
        public MainViewModel(IResourceScope res, AlertService alert)
        {
            Global = res;
            alert.StartMonitoring();
        }

        private IResourceScope Global { get; }

        public IResourceScope Resources { get; set; }

        public bool IsPopupOpen { get; set; }

        public IObservable<AppDurationViewModel> AppDurations { get; set; }

        public BindableCollection<TagDurationViewModel> TagDurations { get; set; }

        public TimeSpan TotalDuration { get; set; }

        public void PopupOpened()
        {
            if (IsPopupOpen)
                PopupClosed();

            IsPopupOpen = true;

            //TODO http://www.introtorx.com/content/v1.0.10621.0/14_HotAndColdObservables.html#HotAndCold try this later
            Resources = Global.Subscope();
            var stats = Resources.Resolve<IAppStatsStreamService>();

            var appUsageStream = stats.GetAppDurations(DateTime.Today).Publish();

            var hasTotalDur = HasDuration.From(
                () => TotalDuration, x => TotalDuration = x);

            var appIncrementor = Resources.Resolve<IDurationIncrementor>();
            var totalDurationIncrementor = Resources.Resolve<IDurationIncrementor>();
            //var tagIncrementor = Resources.Resolve<IDurationIncrementor>();

            /*var repo = Resources.Resolve<IDbRepository>();

            repo.GetIdleDurations(TimeSpan.FromMinutes(1), DateTime.Today)
                .Sum(t => (t.End - t.Start).Ticks)
                .Select(TimeSpan.FromTicks)
                .ObserveOnDispatcher()
                .Subscribe(x => IdleTime = x)
                .ManageUsing(Resources);*/

            AppDurations = appUsageStream
                .Select(x =>
                {
                    //random bug - this is called even though the popup is closed and this has been disposed
                    var appDur = new AppDurationViewModel(x.App);
                    if (!IsPopupOpen) return appDur;

                    x.Duration
                        .Subscribe(d =>
                        {
                            if (!IsPopupOpen) return;
                            appDur.DurationIncrement(d, appIncrementor);
                        })
                        .ManageUsing(Resources);

                    return appDur;
                });

            appUsageStream
                .Select(x => x.Duration)
                .Merge()
                .Subscribe(x =>
                {
                    if (!IsPopupOpen) return;
                    hasTotalDur.DurationIncrement(x, totalDurationIncrementor);
                });

            /*
            stats.GetTagDurations(DateTime.Today)
                .Select(x =>
                {
                    var tagDur = new TagDurationViewModel(x.Tag);

                    x.Duration
                        .Subscribe(d => DurationIncrement(d, tagIncrementor, tagDur))
                        .ManageUsing(Resources);

                    return tagDur;
                })
                
                //TODO after converter and views are made
                .Subscribe(x => TagDurations.Add(x))
                .ManageUsing(Resources)
                */

            appUsageStream.Connect()
                .ManageUsing(Resources);
        }

        public void PopupClosed()
        {
            IsPopupOpen = false;
            Resources.Dispose();
            TotalDuration = TimeSpan.Zero;
            AppDurations = null;
            TagDurations.Clear();
        }

        public void OpenCobalt()
        {
            new Process
            {
                StartInfo = new ProcessStartInfo("Cobalt.exe")
            }.Start();
        }
    }
}