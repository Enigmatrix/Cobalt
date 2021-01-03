using System;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.Communication.Raw;
using Grpc.Core;
using Grpc.Net.Client;

namespace Cobalt.Common.Communication
{
    public class Client
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


        public IObservable<long> AppUpdates()
        {
            return _inner.AppUpdates(new Empty(), new CallOptions())
                .ResponseStream
                .ReadAllAsync()
                .ToObservable()
                .Select(x => x.AppId_);
        }
    }
}
