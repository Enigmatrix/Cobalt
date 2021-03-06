﻿using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipes;
using System.Security.AccessControl;
using System.Security.Principal;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Transmission.Util;
using Newtonsoft.Json;

namespace Cobalt.Common.Transmission
{
    public interface ITransmissionServer
    {
        void Send(MessageBase message);
    }

    public class TransmissionServer : ITransmissionServer
    {
        private readonly List<JsonTextWriter> _broadcasters;
        private readonly List<NamedPipeServerStream> _broadcastingPipes;
        private readonly JsonSerializer _serializer;
        private NamedPipeServerStream _waitingPipe;

        public TransmissionServer()
        {
            _broadcastingPipes = new List<NamedPipeServerStream>();
            _broadcasters = new List<JsonTextWriter>();
            _serializer = Utilities.CreateSerializer();
            SetupPipeForConnection();
        }

        public void Send(MessageBase message)
        {
            lock (_broadcasters)
            {
                for (var i = 0; i < _broadcasters.Count; i++)
                {
                    var writer = _broadcasters[i];
                    try
                    {
                        _serializer.Serialize(writer, message);
                        writer.Flush();
                    }
                    catch (Exception)
                    {
                        try
                        {
                            _broadcastingPipes[i].Disconnect();
                        }
                        catch (Exception)
                        {
                        }

                        _broadcastingPipes[i].Dispose();
                        _broadcastingPipes.RemoveAt(i);
                        _broadcasters.RemoveAt(i);
                        i--;
                    }
                }
            }
        }

        private void SetupPipeForConnection()
        {
            _waitingPipe = new NamedPipeServerStream(
                Utilities.PipeName,
                PipeDirection.Out,
                NamedPipeServerStream.MaxAllowedServerInstances,
                PipeTransmissionMode.Byte,
                PipeOptions.Asynchronous,
                Utilities.ReadWriteSize,
                Utilities.ReadWriteSize,
                CreateSecuritySettings(),
                HandleInheritability.None,
                PipeAccessRights.ChangePermissions);

            _waitingPipe.BeginWaitForConnection(ConnectionCallback, null);
        }

        //implicit threading
        private void ConnectionCallback(IAsyncResult ar)
        {
            lock (_broadcasters)
            {
                var connectedPipe = _waitingPipe;
                connectedPipe.EndWaitForConnection(ar);

                var writer = new JsonTextWriter(new StreamWriter(connectedPipe));
                _broadcasters.Add(writer);
                _broadcastingPipes.Add(connectedPipe);
            }

            SetupPipeForConnection();
        }

        private PipeSecurity CreateSecuritySettings()
        {
            var currentUserSid = $@"{Environment.UserDomainName}\{Environment.UserName}";
            var pipeAccess = new PipeSecurity();
            /*pipeAccess.AddAccessRule(new PipeAccessRule(new SecurityIdentifier(WellKnownSidType.WorldSid, null), PipeAccessRights.FullControl,
                AccessControlType.Allow));*/
            pipeAccess.AddAccessRule(new PipeAccessRule(currentUserSid, PipeAccessRights.FullControl,
                AccessControlType.Allow));
            pipeAccess.AddAccessRule(new PipeAccessRule(new SecurityIdentifier(WellKnownSidType.NetworkSid, null),
                PipeAccessRights.FullControl, AccessControlType.Deny));
            pipeAccess.AddAccessRule(new PipeAccessRule(
                new SecurityIdentifier(WellKnownSidType.NetworkServiceSid, null), PipeAccessRights.FullControl,
                AccessControlType.Deny));
            return pipeAccess;
        }
    }
}