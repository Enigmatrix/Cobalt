using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipes;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Transmission.Util;
using ProtoBuf;

namespace Cobalt.Common.Transmission
{

    public interface ITransmissionServer
    {
        void Send(MessageBase message);
    }

    public class TransmissionServer : ITransmissionServer
    {
        private readonly List<NamedPipeServerStream> _broadcastingPipes;
        private NamedPipeServerStream _waitingPipe;

        public TransmissionServer()
        {
            _broadcastingPipes = new List<NamedPipeServerStream>();
            SetupPipeForConnection();
        }

        public void Send(MessageBase message)
        {
            lock (_broadcastingPipes)
            {
                for (var i = 0; i < _broadcastingPipes.Count; i++)
                {
                    var writer = _broadcastingPipes[i];
                    try
                    {
                        Serializer.NonGeneric.SerializeWithLengthPrefix(
                            writer, message, PrefixStyle.Base128, MessageBase.Index(message));
                        writer.Flush();
                    }
                    catch (Exception)
                    {
                        try
                        {
                            _broadcastingPipes[i].Disconnect();
                        }
                        catch (Exception e)
                        {
                            
                        }

                        _broadcastingPipes[i].Dispose();
                        _broadcastingPipes.RemoveAt(i);
                        i--;
                    }
                }
            }
        }

        private void SetupPipeForConnection()
        {
            //TODO maybe not use .NETStandard, these options are limited (check master branch)
            _waitingPipe = new NamedPipeServerStream(
                Utilities.PipeName,
                PipeDirection.Out,
                NamedPipeServerStream.MaxAllowedServerInstances,
                PipeTransmissionMode.Byte,
                PipeOptions.Asynchronous,
                Utilities.ReadWriteSize,
                Utilities.ReadWriteSize);

            _waitingPipe.BeginWaitForConnection(ConnectionCallback, null);
        }

        //implicit threading
        private void ConnectionCallback(IAsyncResult ar)
        {
            lock (_broadcastingPipes)
            {
                var connectedPipe = _waitingPipe;
                connectedPipe.EndWaitForConnection(ar);

                _broadcastingPipes.Add(connectedPipe);
            }

            SetupPipeForConnection();
        }
    }}
