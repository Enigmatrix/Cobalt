using System;
using System.Reactive.Linq;
using ReactiveUI;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Entities
{
    public abstract class WithDuration<T> : ViewModelBase where T : EntityViewModelBase
    {
        protected WithDuration(T vm, IObservable<TimeSpan> durations)
        {
            Inner = vm;
            Durations = durations;

            Durations
                .Select(x => x.Ticks)
                .Scan((x, y) => x + y)
                .ToPropertyEx(this, x => x.TotalDurationTicks, scheduler: RxApp.MainThreadScheduler);
        }

        public T Inner { get; }

        public IObservable<TimeSpan> Durations { get; set; }

        [ObservableAsProperty] public long TotalDurationTicks { get; }
    }

    public class AppDurationViewModel : WithDuration<AppViewModel>
    {
        public AppDurationViewModel(AppViewModel vm, IObservable<TimeSpan> durations) : base(vm, durations)
        {
        }
    }

    public class TagDurationViewModel : WithDuration<TagViewModel>
    {
        public TagDurationViewModel(TagViewModel vm, IObservable<TimeSpan> durations) : base(vm, durations)
        {
        }
    }

    public class SessionDurationViewModel : WithDuration<SessionViewModel>
    {
        public SessionDurationViewModel(SessionViewModel vm, IObservable<TimeSpan> durations) : base(vm, durations)
        {
        }
    }
}