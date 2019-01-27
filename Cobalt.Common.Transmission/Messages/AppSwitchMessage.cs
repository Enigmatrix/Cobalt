using ProtoBuf;

namespace Cobalt.Common.Transmission.Messages
{
    [ProtoContract]
    public class AppSwitchMessage : MessageBase
    {
        [ProtoMember(1)] public long AppUsageId { get; set; }

        [ProtoMember(2)] public long ActiveAppId { get; set; }
    }
}