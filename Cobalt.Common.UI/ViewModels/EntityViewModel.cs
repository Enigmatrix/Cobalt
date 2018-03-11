using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class EntityViewModel : ViewModelBase
    {
        private int _id;

        public EntityViewModel(Entity entity)
        {
            Entity = entity;
        }

        public int Id
        {
            get => _id;
            set => Set(ref _id, value);
        }

        public Entity Entity { get; }
    }
}