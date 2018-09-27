using System;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagViewModel : EntityViewModel
    {
        private IDurationIncrementor _appIncrementor;
        private BindableCollection<AppViewModel> _taggedApps;
        private IObservable<AppDurationViewModel> _taggedAppsDurationsToday;
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _taggedAppsHourlyChunks;

        public TagViewModel(Tag tag) : base(tag)
        {
            Name = tag.Name;
        }

        public string Name { get; set; }

        public BindableCollection<AppViewModel> TaggedApps
        {
            get
            {
                if (_taggedApps != null) return _taggedApps;

                TaggedApps = new BindableCollection<AppViewModel>();
                Repository.GetAppsWithTag((Tag) Entity).Subscribe(x => TaggedApps.Add(new AppViewModel(x)));
                return _taggedApps;
            }
            set
            {
                Set(ref _taggedApps, value);
                _taggedApps.CollectionChanged += (o, e) =>
                {
                    //refresh all bindings with new data
                    _taggedAppsDurationsToday = null;
                    _appIncrementor = null;
                    _taggedAppsHourlyChunks = null;
                    Refresh();
                };
            }
        }


        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> TaggedAppsHourlyChunks =>
            _taggedAppsHourlyChunks ?? (_taggedAppsHourlyChunks = AppStats.GetChunkedAppDurations((Tag) Entity,
                TimeSpan.FromHours(1),
                d => d.Date.AddHours(d.Hour), DateTime.Today));

        private IDurationIncrementor AppIncrementor =>
            _appIncrementor ?? (_appIncrementor = Resources.Resolve<IDurationIncrementor>());

        public IObservable<AppDurationViewModel> TaggedAppDurationsToday =>
            _taggedAppsDurationsToday ?? (_taggedAppsDurationsToday = AppStats
                .GetTaggedAppDurations((Tag) Entity, DateTime.Today)
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d => { appDur.DurationIncrement(d, AppIncrementor); })
                        .ManageUsing(Resources);

                    return appDur;
                }));
    }
}