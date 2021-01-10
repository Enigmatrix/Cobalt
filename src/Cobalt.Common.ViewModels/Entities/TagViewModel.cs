#pragma warning disable CS8618 // Non-nullable field must contain a non-null value when exiting constructor. Consider declaring as nullable.
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Entities
{
    public class TagViewModel : MutableEntityViewModelBase<Tag>
    {
        public TagViewModel(Tag tag, IEntityManager manager, IDatabase db) : base(tag, manager, db)
        {
            Id = tag.Id;
            UpdateFromEntity(tag);
        }

        [Reactive] public string Name { get; set; }
        [Reactive] public string Description { get; set; }
        [Reactive] public string Color { get; set; }

        public sealed override void UpdateFromEntity(Tag tag)
        {
            Name = tag.Name;
            Description = tag.Description;
            Color = tag.Color;
        }

        public override void Save()
        {
            // TODO Save to database
            Manager.InformTagUpdate(Id);
        }
    }
}
#pragma warning restore CS8618