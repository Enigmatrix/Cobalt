using System;
using System.Text.RegularExpressions;
using Cobalt.Common.Communication;
using Cobalt.Common.Communication.Raw;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Utils;

namespace Cobalt.Common.ViewModels.Entities
{
    public class EntityNotFoundException<T> : Exception
    {
        public override string Message => $"Entity of type {typeof(T)} not found in operation";
    }
    
    public interface IEntityManager : IDisposable
    {
        AppViewModel GetApp(long id);
        TagViewModel GetTag(long id);
        SessionViewModel GetSession(long id);
        UsageViewModel GetUsage(long id);
        AppViewModel GetApp(App app);
        TagViewModel GetTag(Tag tag);
        SessionViewModel GetSession(Session session);
        UsageViewModel GetUsage(Usage usage);
    }

    public class EntityManager : IEntityManager
    {
        private readonly IDatabase _db;
        private readonly IDisposable _entityUpdatesSub;

        public EntityManager(IDatabase db, IClient client)
        {
            _db = db;

            _entityUpdatesSub = client.EntityUpdates()
                .Subscribe(entity =>
                {
                    switch (entity.Etype)
                    {
                        case UpdatedEntity.Types.EntityType.App:
                            Apps.Get(entity.Id)?.Update(_db.FindApp(entity.Id));
                            break;
                        case UpdatedEntity.Types.EntityType.Tag:
                            throw new NotImplementedException();
                        case UpdatedEntity.Types.EntityType.Alert:
                            throw new NotImplementedException();
                        default:
                            throw new ArgumentOutOfRangeException(nameof(entity.Etype));
                    }
                });
        }

        public WeakValueCache<long, AppViewModel> Apps { get; } = new();
        public WeakValueCache<long, SessionViewModel> Sessions { get; } = new();
        public WeakValueCache<long, UsageViewModel> Usages { get; } = new();
        public WeakValueCache<long, TagViewModel> Tags { get; } = new();

        public AppViewModel GetApp(long id)
        {
            var vm = Apps[id];
            if (vm != null) return vm;

            var m = _db.FindApp(id) ?? throw new EntityNotFoundException<App>();

            return GetApp(m);
        }

        public TagViewModel GetTag(long id)
        {
            var vm = Tags[id];
            if (vm != null) return vm;

            var m = _db.FindTag(id) ?? throw new EntityNotFoundException<Tag>();

            return GetTag(m);
        }

        public SessionViewModel GetSession(long id)
        {
            var vm = Sessions[id];
            if (vm != null) return vm;

            var m = _db.FindSession(id) ?? throw new EntityNotFoundException<Session>();

            return GetSession(m);
        }

        public UsageViewModel GetUsage(long id)
        {
            var vm = Usages[id];
            if (vm != null) return vm;

            var m = _db.FindUsage(id) ?? throw new EntityNotFoundException<Usage>();

            return GetUsage(m);
        }

        public AppViewModel GetApp(App app)
        {
            var vm = new AppViewModel(app, this);
            Apps.Set(app.Id, vm);

            return vm;
        }

        public TagViewModel GetTag(Tag tag)
        {
            var vm = new TagViewModel(tag, this);
            Tags.Set(tag.Id, vm);

            return vm;
        }

        public SessionViewModel GetSession(Session session)
        {
            var vm = new SessionViewModel(session, this);
            Sessions.Set(session.Id, vm);

            return vm;
        }

        public UsageViewModel GetUsage(Usage usage)
        {
            var vm = new UsageViewModel(usage, this);
            Usages.Set(usage.Id, vm);

            return vm;
        }

        public void Dispose()
        {
            _entityUpdatesSub.Dispose();
        }
    }
}