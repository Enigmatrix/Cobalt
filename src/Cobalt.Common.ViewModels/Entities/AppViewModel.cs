using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Microsoft.FSharp.Core;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Entities
{
    public class AppViewModel : MutableEntityViewModelBase<App>
    {
        public AppViewModel(App app, IEntityManager manager, IDatabase db) : base(app, manager, db)
        {
            Id = app.Id;
            UpdateFromEntity(app);
            Identity = app.Identity;
        }

        [Reactive] public string? Name { get; set; }

        [Reactive] public string? Description { get; set; }

        [Reactive] public string? Color { get; set; }

        // TODO Icon

        public AppIdentity Identity { get; }

        public sealed override void UpdateFromEntity(App app)
        {
            Name = ValueOption.ToObj(app.Name);
            Description = ValueOption.ToObj(app.Description);
            Color = ValueOption.ToObj(app.Color);
        }

        public override void Save()
        {
            // TODO Save to database
            Manager.InformAppUpdate(Id);
        }
    }
}