using System;
using Cobalt.Common.Data;
using Cobalt.Common.Utils;

namespace Cobalt.Common.ViewModels.Entities
{
    public interface IEntityManager
    {
        AppViewModel GetApp(long id);
        TagViewModel GetTag(long id);
        SessionViewModel GetSession(long id);
        UsageViewModel GetUsage(long id);
    }

    public class EntityManager : IEntityManager
    {
        private readonly IDatabase _db;

        public EntityManager(IDatabase db)
        {
            _db = db;
        }

        public WeakValueCache<long, AppViewModel> Apps { get; } = new();
        public WeakValueCache<long, SessionViewModel> Sessions { get; } = new();

        public AppViewModel GetApp(long id)
        {
            var vm = Apps[id];
            if (vm != null) return vm;

            var m = _db.FindApp(id);
            if (m == null) throw new NullReferenceException(nameof(m)); // TODO use different exception

            vm = new AppViewModel(m, this);
            Apps.Set(id, vm);

            return vm;
        }

        public TagViewModel GetTag(long id)
        {
            throw new NotImplementedException();
        }

        public SessionViewModel GetSession(long id)
        {
            var vm = Sessions[id];
            if (vm != null) return vm;

            var m = _db.FindSession(id);
            if (m == null) throw new NullReferenceException(nameof(m)); // TODO use different exception

            vm = new SessionViewModel(m, this);
            Sessions.Set(id, vm);

            return vm;
        }

        public UsageViewModel GetUsage(long id)
        {
            throw new NotImplementedException();
        }
    }
}