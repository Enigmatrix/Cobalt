using System;
using System.Collections.Generic;
using System.Text;
using ProtoBuf;

namespace Cobalt.Common.Transmission.Messages
{
    public enum ChangeType
    {
        Add,
        Update,
        Remove
    }

    public enum EntityType
    {
        Alert,
        App
    }

    [ProtoContract]
    public class EntityChangeMessage : MessageBase
    {
        [ProtoMember(1)]
        public ChangeType ChangeType { get; set; }

        [ProtoMember(2)]
        public long EntityId { get; set; }

        [ProtoMember(3)]
        public EntityType EntityType { get; set; }
    }
}
