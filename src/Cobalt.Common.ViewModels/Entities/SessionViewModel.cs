using Cobalt.Common.Data.Entities;
using Microsoft.FSharp.Core;

namespace Cobalt.Common.ViewModels.Entities
{
    public class SessionViewModel : EntityViewModelBase<Session>
    {
        public SessionViewModel(Session session, IEntityManager mgr) : base(mgr)
        {
            Id = session.Id;
            Title = session.Title;
            Arguments = ValueOption.ToObj(session.Arguments);
            AppId = session.AppId;
        }

        public string Title { get; }
        public string? Arguments { get; }
        public long AppId { get; }
        public AppViewModel App => _mgr.GetApp(AppId);
    }
}