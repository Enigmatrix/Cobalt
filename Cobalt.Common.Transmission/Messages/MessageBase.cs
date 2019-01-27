using System;
using System.Collections.Generic;

namespace Cobalt.Common.Transmission.Messages
{
    public abstract class MessageBase
    {
        private static readonly IDictionary<int, Type> MessageTypeLookup = new Dictionary<int, Type>
        {
            [1] = typeof(AppSwitchMessage),
            [2] = typeof(EntityChangeMessage)
        };

        private static readonly IDictionary<string, int> MessageIndexLookup = new Dictionary<string, int>
        {
            [typeof(AppSwitchMessage).Name] = 1,
            [typeof(EntityChangeMessage).Name] = 2
        };

        public static Type MessageTypeResolver(int i)
        {
            return MessageTypeLookup[i];
        }

        internal static int Index(MessageBase message)
        {
            return MessageIndexLookup[message.GetType().Name];
        }
    }
}