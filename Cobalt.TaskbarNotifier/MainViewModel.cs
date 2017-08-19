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
        BindableCollection<AppDurationViewModel> AppDurations { get; }
        BindableCollection<TagDurationViewModel> TagDurations { get; }
    }

    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private BindableCollection<AppDurationViewModel> _appDurations = new BindableCollection<AppDurationViewModel>();

        private BindableCollection<TagDurationViewModel> _tagDurations = new BindableCollection<TagDurationViewModel>();

        public MainViewModel(IResourceScope res)
        {
            Global = res;
        }

        private IResourceScope Global { get; }
        private IResourceScope Current { get; set; }

        public BindableCollection<AppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }

        public BindableCollection<TagDurationViewModel> TagDurations
        {
            get => _tagDurations;
            set => Set(ref _tagDurations, value);
        }

        public void PopupOpened()
        {
            Current = Global.Subscope();
            var stats = Current.Resolve<IAppStatsStreamService>();
            var appIncrementor = Current.Resolve<IDurationIncrementor>();
            //var tagIncrementor = Current.Resolve<IDurationIncrementor>();

            /*
            AppDurations = new BindableCollection<AppDurationViewModel>();
            TagDurations = new BindableCollection<TagDurationViewModel>();
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
                                appIncrementor.Increment(appDur);
                            }
                            else
                            {
                                appIncrementor.Release();
                                appDur.Duration += d.Value;
                            }
                        })
                        .ManageUsing(Current);

                    return appDur;
                })
                .Subscribe(x => AppDurations.Add(x))
                .ManageUsing(Current);

            stats.GetTagDurations(DateTime.Today)
                .Select(x =>
                {
                    var tagDur = new TagDurationViewModel(x.Tag);

                    x.Duration
                        .Subscribe(d =>
                        {
                            if (d is null)
                            {
                                //handle new tag started here
                                //tagIncrementor.Increment(tagDur);
                            }
                            else
                            {
                                //tagIncrementor.Release();
                                tagDur.Duration += d.Value;
                            }
                        })
                        .ManageUsing(Current);

                    return tagDur;
                })
                //TODO after converter and views are made
                /*.Subscribe(x => TagDurations.Add(x))
                .ManageUsing(Current)*/;
        }

        public void PopupClosed()
        {
            AppDurations.Clear();
            TagDurations.Clear();
            Current.Dispose();
        }
    }
}