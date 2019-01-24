using Cobalt.Common.Data.Entities;
using ProtoBuf;

namespace Cobalt.Common.Transmission.Messages
{
    [ProtoContract]
    public class AppUsageEndMessage : MessageBase
    {
        [ProtoMember(1)] public long AppUsageId { get; set; }

        [ProtoMember(2)] public long ActiveAppUsageId { get; set; }
    }
}