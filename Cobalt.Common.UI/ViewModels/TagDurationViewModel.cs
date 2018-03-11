using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagDurationViewModel : ViewModelBase, IHasDuration
    {
        private TimeSpan _duration;
        private TagViewModel _tag;

        public TagDurationViewModel(Tag tag) : this(tag, TimeSpan.Zero)
        {
        }

        public TagDurationViewModel(Tag tag, TimeSpan span)
        {
            Tag = new TagViewModel(tag);
            Duration = span;
        }

        public TagViewModel Tag
        {
            get => _tag;
            set => Set(ref _tag, value);
        }

        public TimeSpan Duration
        {
            get => _duration;
            set => Set(ref _duration, value);
        }
    }
}