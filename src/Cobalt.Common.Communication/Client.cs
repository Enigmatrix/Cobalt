using System;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.Communication.Raw;
using Grpc.Core;
using Grpc.Net.Client;

namespace Cobalt.Common.Communication
{
    public interface IClient
    {
        IObservable<UsageSwitch> Usages();
        IObservable<UpdatedEntity> EntityUpdates();
    }

    public class Client : IClient
    {
        private readonly Relay.RelayClient _inner;

        public Client()
        {
            var channel = GrpcChannel.ForAddress("http://locahost:10691");
            _inner = new Relay.RelayClient(channel);
        }

        public IObservable<UsageSwitch> Usages()
        {
            return _inner.Usages(new Empty(), new CallOptions())
                .ResponseStream
                .ReadAllAsync()
                .ToObservable();
        }


        public IObservable<UpdatedEntity> EntityUpdates()
        {
            return _inner.EntityUpdates(new Empty(), new CallOptions())
                .ResponseStream
                .ReadAllAsync()
                .ToObservable();
        }
    }
}