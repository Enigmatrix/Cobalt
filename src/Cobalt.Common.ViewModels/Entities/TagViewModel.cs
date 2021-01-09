using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.ViewModels.Entities
{
    public class TagViewModel : EntityViewModelBase<Tag>
    {
        public TagViewModel(Tag tag, IEntityManager mgr) : base(mgr)
        {
            Id = tag.Id;
            Name = tag.Name;
            Description = tag.Description;
            Color = tag.Color;
        }

        public string Name { get; set; }
        public string Description { get; set; }
        public string Color { get; set; }
    }
}