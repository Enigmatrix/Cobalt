using System;
using System.Diagnostics;
using System.IO;
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
        BindableCollection<IAppDurationViewModel> AppDurations { get; }
        BindableCollection<ITagDurationViewModel> TagDurations { get; }
        TimeSpan TotalDuration { get; set; }
        void PopupOpened();
        void PopupClosed();
        void OpenCobalt();
    }

    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private BindableCollection<IAppDurationViewModel> _appDurations =
            new BindableCollection<IAppDurationViewModel>();

        private bool _isPopupOpen;

        private BindableCollection<ITagDurationViewModel> _tagDurations =
            new BindableCollection<ITagDurationViewModel>();

        private TimeSpan _totalDuration;


        public MainViewModel(IResourceScope res)
        {
            Global = res;
        }

        private IResourceScope Global { get; }
        private IResourceScope Current { get; set; }

        public bool IsPopupOpen
        {
            get => _isPopupOpen;
            set => Set(ref _isPopupOpen, value);
        }

        public BindableCollection<IAppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }

        public BindableCollection<ITagDurationViewModel> TagDurations
        {
            get => _tagDurations;
            set => Set(ref _tagDurations, value);
        }

        public TimeSpan TotalDuration
        {
            get => _totalDuration;
            set => Set(ref _totalDuration, value);
        }

        public void PopupOpened()
        {
            if (IsPopupOpen)
                PopupClosed();

            IsPopupOpen = true;

            //TODO http://www.introtorx.com/content/v1.0.10621.0/14_HotAndColdObservables.html#HotAndCold try this later
            Current = Global.Subscope();
            var stats = Current.Resolve<IAppStatsStreamService>();

            var appUsageStream = stats.GetAppDurations(DateTime.Today).Publish();

            var hasTotalDur = HasDuration.From(
                () => TotalDuration, x => TotalDuration = x);

            var appIncrementor = Current.Resolve<IDurationIncrementor>();
            var totalDurationIncrementor = Current.Resolve<IDurationIncrementor>();
            //var tagIncrementor = Current.Resolve<IDurationIncrementor>();

            /*var repo = Current.Resolve<IDbRepository>();

            repo.GetIdleDurations(TimeSpan.FromMinutes(1), DateTime.Today)
                .Sum(t => (t.End - t.Start).Ticks)
                .Select(TimeSpan.FromTicks)
                .ObserveOnDispatcher()
                .Subscribe(x => IdleTime = x)
                .ManageUsing(Current);*/

            appUsageStream
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
                        .ManageUsing(Current);

                    return appDur;
                })
                .ObserveOnDispatcher()
                .Subscribe(x =>
                    AppDurations.Add(x));

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
                        .ManageUsing(Current);

                    return tagDur;
                })
                
                //TODO after converter and views are made
                .Subscribe(x => TagDurations.Add(x))
                .ManageUsing(Current)
                */

            appUsageStream.Connect()
                .ManageUsing(Current);
        }

        public void PopupClosed()
        {
            IsPopupOpen = false;
            Current.Dispose();
            TotalDuration = TimeSpan.Zero;
            AppDurations.Clear();
            TagDurations.Clear();
        }

        public void OpenCobalt()
        {
            new Process {
                StartInfo = new ProcessStartInfo("Cobalt.exe")
            }.Start();
        }
    }
}