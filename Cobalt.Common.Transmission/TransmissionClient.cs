using System;
using System.IO;
using System.IO.Pipes;
using System.Threading;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Transmission.Util;
using Newtonsoft.Json;

namespace Cobalt.Common.Transmission
{
    public interface ITransmissionClient : IDisposable
    {
        event EventHandler<MessageReceivedArgs> MessageReceived;
    }

    public class TransmissionClient : ITransmissionClient
    {
        private readonly NamedPipeClientStream _pipe;
        private bool _keepAlive;
        private readonly Thread _listeningThread;

        public TransmissionClient()
        {
            _pipe = new NamedPipeClientStream(
                Utilities.LocalComputer,
                Utilities.PipeName,
                PipeDirection.In,
                PipeOptions.Asynchronous);
            _pipe.Connect(Utilities.PipeConnectionTimeout);
            //_pipe.ReadMode = PipeTransmissionMode.Message;
            _keepAlive = true;

            var reader = new JsonTextReader(new StreamReader(_pipe)) {SupportMultipleContent = true};
            var serializer = Utilities.CreateSerializer();

            _listeningThread = new Thread(() =>
            {
                while (_keepAlive)
                    MessageReceived?.Invoke(this,
                        new MessageReceivedArgs(serializer.Deserialize<MessageBase>(reader)));
            });
            _listeningThread.Start();
        }

        public void Dispose()
        {
            _keepAlive = false;
            _pipe.Dispose();
        }

        public event EventHandler<MessageReceivedArgs> MessageReceived;
    }
}