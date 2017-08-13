using System;
using Cobalt.Common.Transmission.Messages;

namespace Cobalt.Common.Transmission
{
    public class MessageReceivedArgs : EventArgs
    {
        public MessageReceivedArgs(MessageBase receivedMessage)
        {
            Message = receivedMessage;
        }

        public MessageBase Message { get; set; }
    }
}