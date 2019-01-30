using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Data.Repositories;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using DynamicData;
using ChangeType = Cobalt.Common.Transmission.Messages.ChangeType;

namespace Cobalt.Common.Services
{
    public interface IEntityService
    {
        IObservable<IChangeSet<Alert>> GetAlerts();
        IObservable<IChangeSet<Reminder>> GetRemindersForAlert(Alert alert);
    }

    public class EntityService : IEntityService
    {
        private ITransmissionClient Client { get; }
        private IDbRepository Repository { get; }

        public EntityService(ITransmissionClient client, IDbRepository repo)
        {
            Client = client;
            Repository = repo;
        }

        public IObservable<IChangeSet<Alert>> GetAlerts()
        {
            return Get(EntityType.Alert, () => Repository.Get<Alert>());
        }

        public IObservable<IChangeSet<Reminder>> GetRemindersForAlert(Alert alert)
        {
            return Get(EntityType.Reminder, () => Repository.GetRemindersForAlert(alert));
        }

        private IObservable<IChangeSet<T>> Get<T>(EntityType type, Func<IObservable<T>> getAll)
            where T : Entity
        {
            return ObservableChangeSet.Create<T>(obs =>
            {
                obs.AddRange(getAll().ToEnumerable());

                var changes = Client.Messages<EntityChangeMessage>()
                    .Where(x => x.EntityType == type)
                    //TODO make a equality comparer?
                    .Subscribe(_ => obs.EditDiff(getAll().ToEnumerable()));

                return new CompositeDisposable(changes);
            });
        }
    }
}
