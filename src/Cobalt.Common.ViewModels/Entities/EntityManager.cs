using System;
using System.Collections.Generic;
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
        AlertViewModel GetAlert(long id);

        AppViewModel GetApp(App app);
        TagViewModel GetTag(Tag tag);
        SessionViewModel GetSession(Session session);
        UsageViewModel GetUsage(Usage usage);
        AlertViewModel GetAlert(Alert alert);

        void InformAppUpdate(long id);
        void InformTagUpdate(long id);
        void InformAlertUpdate(long id);
    }

    public class EntityManager : IEntityManager
    {
        private readonly IClient _client;
        private readonly IDatabase _db;
        private readonly IDisposable _entityUpdatesSub;

        private readonly HashSet<long> _updatedAlerts = new();
        private readonly HashSet<long> _updatedApps = new();
        private readonly HashSet<long> _updatedTags = new();

        public EntityManager(IDatabase db, IClient client)
        {
            _db = db;
            _client = client;

            _entityUpdatesSub = client.EntityUpdates()
                .Subscribe(entity =>
                {
                    switch (entity.Etype)
                    {
                        case UpdatedEntity.Types.EntityType.App:
                            if (!_updatedApps.Remove(entity.Id)) break;
                            Apps.Get(entity.Id)?.UpdateFromEntity(_db.FindApp(entity.Id));
                            break;
                        case UpdatedEntity.Types.EntityType.Tag:
                            if (!_updatedTags.Remove(entity.Id)) break;
                            Tags.Get(entity.Id)?.UpdateFromEntity(_db.FindTag(entity.Id));
                            break;
                        case UpdatedEntity.Types.EntityType.Alert:
                            if (!_updatedAlerts.Remove(entity.Id)) break;
                            Alerts.Get(entity.Id)?.UpdateFromEntity(_db.FindAlert(entity.Id));
                            break;
                        default:
                            throw new ArgumentOutOfRangeException(nameof(entity.Etype));
                    }
                });
        }

        public WeakValueCache<long, AlertViewModel> Alerts { get; } = new();
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

        public AlertViewModel GetAlert(long id)
        {
            var vm = Alerts[id];
            if (vm != null) return vm;

            var m = _db.FindAlert(id) ?? throw new EntityNotFoundException<Alert>();

            return GetAlert(m);
        }

        public AppViewModel GetApp(App app)
        {
            var vm = new AppViewModel(app, this, _db);
            Apps.Set(app.Id, vm);

            return vm;
        }

        public TagViewModel GetTag(Tag tag)
        {
            var vm = new TagViewModel(tag, this, _db);
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

        public AlertViewModel GetAlert(Alert alert)
        {
            var vm = new AlertViewModel(alert, this, _db);
            Alerts.Set(alert.Id, vm);

            return vm;
        }

        public void InformAppUpdate(long id)
        {
            _updatedApps.Add(id);
            _client.InformEntityUpdate(new UpdatedEntity {Etype = UpdatedEntity.Types.EntityType.App, Id = id});
        }

        public void InformTagUpdate(long id)
        {
            _updatedTags.Add(id);
            _client.InformEntityUpdate(new UpdatedEntity {Etype = UpdatedEntity.Types.EntityType.Tag, Id = id});
        }

        public void InformAlertUpdate(long id)
        {
            _updatedAlerts.Add(id);
            _client.InformEntityUpdate(new UpdatedEntity {Etype = UpdatedEntity.Types.EntityType.Alert, Id = id});
        }

        public void Dispose()
        {
            _entityUpdatesSub.Dispose();
        }
    }
}