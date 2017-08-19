using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public interface ITagDurationViewModel : IViewModel, IHasDuration
    {
        Tag Tag { get; set; }
    }

    public class TagDurationViewModel : ViewModelBase, ITagDurationViewModel
    {
        private TimeSpan _duration;
        private Tag _tag;

        public TagDurationViewModel(Tag tag) : this(tag, TimeSpan.Zero)
        {
        }

        public TagDurationViewModel(Tag tag, TimeSpan span)
        {
            Tag = tag;
            Duration = span;
        }

        public TimeSpan Duration
        {
            get => _duration;
            set => Set(ref _duration, value);
        }

        public Tag Tag
        {
            get => _tag;
            set => Set(ref _tag, value);
        }
    }
}