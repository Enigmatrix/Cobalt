using System;
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
    }

    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private BindableCollection<IAppDurationViewModel> _appDurations =
            new BindableCollection<IAppDurationViewModel>();
        private BindableCollection<ITagDurationViewModel> _tagDurations =
            new BindableCollection<ITagDurationViewModel>();
        private bool _isPopupOpen;


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

        public void PopupOpened()
        {
            IsPopupOpen = true;

            Current = Global.Subscope();
            var stats = Current.Resolve<IAppStatsStreamService>();
            var appIncrementor = Current.Resolve<IDurationIncrementor>();
            //var tagIncrementor = Current.Resolve<IDurationIncrementor>();
            //TODO ARGh if tray is clicked is repeatedly clicked, no appswitches are detected initially
            //TODO this is since the transition is from taskbarICON of THIS APP -> POPUP of THIS APP (not winexplorer->this app)
            //TODO but this is a rare enough scenario (what kind of user presses the same thing twice amirite....shit), just screw it

            stats.GetAppDurations(DateTime.Today)
                .Select(x =>
                {
                    //random bug - this is called even though the popup is closed and this has been disposed
                    //so this is a workaroudn
                    if (!IsPopupOpen) return new AppDurationViewModel(x.App);

                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d =>
                        {
                            //lock (Current)
                            if (!IsPopupOpen) return;
                            HandleDuration(d, appIncrementor, appDur);
                        })
                        .ManageUsing(Current);

                    return appDur;
                })
                .ObserveOnDispatcher()
                .Subscribe(x =>
                    AppDurations.Add(x))
                .ManageUsing(Current);
            /*
            stats.GetTagDurations(DateTime.Today)
                .Select(x =>
                {
                    var tagDur = new TagDurationViewModel(x.Tag);

                    x.Duration
                        .Subscribe(d => HandleDuration(d, tagIncrementor, tagDur))
                        .ManageUsing(Current);

                    return tagDur;
                })
                
                //TODO after converter and views are made
                .Subscribe(x => TagDurations.Add(x))
                .ManageUsing(Current)
                */
        }

        public void PopupClosed()
        {
            IsPopupOpen = false;
            Current.Dispose();
            AppDurations.Clear();
            TagDurations.Clear();
        }

        private void HandleDuration(TimeSpan? d, IDurationIncrementor incrementor, IHasDuration hasDur)
        {
            if (d is null)
            {
                //handle new app/tag started here
                incrementor.Increment(hasDur);
            }
            else
            {
                incrementor.Release();
                hasDur.Duration += d.Value;
            }
        }
    }
}