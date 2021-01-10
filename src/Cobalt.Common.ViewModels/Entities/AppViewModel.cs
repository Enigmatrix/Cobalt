using Cobalt.Common.Data.Entities;
using Microsoft.FSharp.Core;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Entities
{
    public class AppViewModel : EntityViewModelBase<App>
    {
        public AppViewModel(App app, IEntityManager mgr) : base(mgr)
        {
            Id = app.Id;
            Update(app);
            Identity = app.Identity;
        }

        [Reactive] public string? Name { get; set; }

        [Reactive] public string? Description { get; set; }

        [Reactive] public string? Color { get; set; }

        public AppIdentity Identity { get; }

        public void Update(App app)
        {
            Name = ValueOption.ToObj(app.Name);
            Description = ValueOption.ToObj(app.Description);
            Color = ValueOption.ToObj(app.Color);
        }
    }
}