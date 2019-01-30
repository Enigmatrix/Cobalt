using System;
using System.IO.Pipes;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Threading;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Transmission.Util;
using ProtoBuf;
using Serilog;

namespace Cobalt.Common.Transmission
{
    public interface ITransmissionClient : IDisposable
    {
        IObservable<MessageBase> Messages();
        IObservable<T> Messages<T>() where T : MessageBase;
    }

    public class TransmissionClient : ITransmissionClient
    {
        private readonly Thread _listeningThread;
        private readonly Subject<MessageBase> _messages;
        private readonly NamedPipeClientStream _pipe;
        private bool _keepAlive;

        public TransmissionClient()
        {
            _messages = new Subject<MessageBase>();
            _pipe = new NamedPipeClientStream(
                Utilities.LocalComputer,
                Utilities.PipeName,
                PipeDirection.In,
                PipeOptions.Asynchronous);
            _pipe.Connect(Utilities.PipeConnectionTimeout);
            //_pipe.ReadMode = PipeTransmissionMode.Message;
            _keepAlive = true;

            _listeningThread = new Thread(() =>
            {
                while (_keepAlive)
                {
                    try
                    {
                        if (Serializer.NonGeneric.TryDeserializeWithLengthPrefix(_pipe, PrefixStyle.Base128,
                            MessageBase.MessageTypeResolver, out var msg))
                            _messages.OnNext((MessageBase) msg);
                    }
                    catch(Exception e)
                    {
                        Log.Fatal(e, "Pipe read exception");
                    }
                }
            });
            _listeningThread.Start();
        }

        public IObservable<MessageBase> Messages() => _messages;

        public IObservable<T> Messages<T>()
            where T : MessageBase
        {
            return Messages().OfType<T>();
        }

        public void Dispose()
        {
            _keepAlive = false;
            _pipe.Dispose();
            _listeningThread.Abort();
        }
    }
}