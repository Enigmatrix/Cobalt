using System;
using System.IO.Pipes;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Threading;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Transmission.Util;
using Newtonsoft.Json;
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
                try
                {
                    while (_keepAlive)
                        using (var reader = new JsonTextReader(streamReader) {CloseInput = false})
                        {
                            SingalMessageReceived(serializer.Deserialize<MessageBase>(reader));
                        }
                }
                catch (Exception e)
                {
                    Log.Error(e, "Error on client listener thread: ");
                }
            });
            _listeningThread.Start();
        }

        public IObservable<MessageBase> Messages()
        {
            return _messages;
        }

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
