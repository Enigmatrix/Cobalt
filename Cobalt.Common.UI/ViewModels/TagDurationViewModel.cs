using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class TagDurationViewModel : ViewModelBase, IHasDuration
    {
        public TagDurationViewModel(Tag tag) : this(tag, TimeSpan.Zero)
        {
        }

        public TagDurationViewModel(Tag tag, TimeSpan span)
        {
            Tag = new TagViewModel(tag);
            Duration = span;
        }

        public TagViewModel Tag { get; set; }

        public TimeSpan Duration { get; set; }
    }
}