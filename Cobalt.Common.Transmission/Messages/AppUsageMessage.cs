using System;
using System.Collections.Generic;
using System.Text;
using Cobalt.Common.Data.Entities;
using ProtoBuf;

namespace Cobalt.Common.Transmission.Messages
{
    [ProtoContract]
    public class AppUsageEndMessage : MessageBase
    {
        [ProtoMember(1)]
        public long AppUsageId { get; set; }

        [ProtoMember(2)]
        public long[] ActiveAppIds { get; set; }

        [ProtoMember(3)]
        public AppUsageType UsageType { get; set; }
    }
}
