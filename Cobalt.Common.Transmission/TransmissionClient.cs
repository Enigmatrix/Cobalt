using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipes;
using System.Reactive.Subjects;
using System.Text;
using System.Threading;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Transmission.Util;
using ProtoBuf;

namespace Cobalt.Common.Transmission
{

    public interface ITransmissionClient : IDisposable
    {
        IObservable<MessageBase> Messages { get; }
    }

    public class TransmissionClient : ITransmissionClient
    {
        private readonly Thread _listeningThread;
        private readonly NamedPipeClientStream _pipe;
        private readonly Subject<MessageBase> _messages;
        private bool _keepAlive;

        public IObservable<MessageBase> Messages => _messages;

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

            Serializer.Deserialize<MessageBase>(_pipe);

            _listeningThread = new Thread(() =>
            {
                while (_keepAlive)
                    _messages.OnNext(Serializer.Deserialize<MessageBase>(_pipe));
            });
            _listeningThread.Start();
        }

        public void Dispose()
        {
            _keepAlive = false;
            _pipe.Dispose();
        }
    }}
