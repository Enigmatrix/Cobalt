using System.IO;
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

        [Reactive] public byte[]? Icon { get; set; }

        public AppIdentity Identity { get; }

        public sealed override void UpdateFromEntity(App app)
        {
            Name = ValueOption.ToObj(app.Name);
            Description = ValueOption.ToObj(app.Description);
            Color = ValueOption.ToObj(app.Color);

            if (Name == null || Description == null || Color == null) return;

            using var icon = Database.AppIcon(Id);
            using var mem = new MemoryStream(new byte[icon.Length]);
            icon.CopyTo(mem);
            Icon = mem.ToArray();
        }

        public override void Save()
        {
            Database.UpdateApp(new App
            {
                Id = Id,
                Color = ValueOption.OfObj(Color),
                Name = ValueOption.OfObj(Name),
                Description = ValueOption.OfObj(Description),
                Identity = Identity
            });
            Manager.InformAppUpdate(Id);
        }
    }
}