using System;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Data.Repositories;
using Cobalt.Common.Transmission;

namespace Cobalt.Common.Services
{
    public interface IStatisticsService
    {
        IObservable<TimeSpan> GetAppDuration(App app, DateTime? start = null, DateTime? end = null);
        IObservable<TimeSpan> GetTagDuration(Tag tag, DateTime? start = null, DateTime? end = null);
    }

    public class StatisticsService : IStatisticsService
    {
        public StatisticsService(ITransmissionClient client, IDbRepository repo)
        {
            Client = client;
            Repository = repo;
        }

        private ITransmissionClient Client { get; }
        private IDbRepository Repository { get; }


        public IObservable<TimeSpan> GetAppDuration(App app, DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<TimeSpan> GetTagDuration(Tag tag, DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }
    }
}