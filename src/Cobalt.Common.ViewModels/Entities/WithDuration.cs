using System;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Entities
{
    public interface IHasDuration
    {
        TimeSpan Duration { get; }
    }

    public abstract class WithDuration<T> : ViewModelBase, IHasDuration where T : EntityViewModelBase
    {
        protected WithDuration(T vm)
        {
            Inner = vm;
        }

        public T Inner { get; }

        [Reactive] public TimeSpan Duration { get; set; }
    }

    public class AppDurationViewModel : WithDuration<AppViewModel>
    {
        public AppDurationViewModel(AppViewModel vm) : base(vm)
        {
        }
    }

    public class TagDurationViewModel : WithDuration<TagViewModel>
    {
        public TagDurationViewModel(TagViewModel vm) : base(vm)
        {
        }
    }

    public class SessionDurationViewModel : WithDuration<SessionViewModel>
    {
        public SessionDurationViewModel(SessionViewModel vm) : base(vm)
        {
        }
    }
}