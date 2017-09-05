using System;
using System.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Transmission.Messages;
using Newtonsoft.Json;

namespace Cobalt.Common.Transmission.Util
{
    public static class Utilities
    {
        public static readonly string LocalComputer = ".";
        public static readonly string PipeName = "CobaltNamedPipeXD";
        public static readonly int PipeConnectionTimeout = 2000;
        public static readonly int ReadWriteSize = 0;

        public static JsonSerializer CreateSerializer()
        {
            return JsonSerializer.Create(new JsonSerializerSettings
            {
                TypeNameHandling = TypeNameHandling.Objects,
                //avoid Remote Code Execution by only whitelisting the types we are using
                SerializationBinder = new WhitelistSerializationBinder(
                    TypesInNamespaces(
                        typeof(Entity).Namespace,
                        typeof(MessageBase).Namespace)),
#if DEBUG
                TraceWriter = new DebugTraceWriter(),
#endif
            });
        }

        public static Type[] TypesInNamespaces(params string[] namespaces)
        {
            return AppDomain.CurrentDomain.GetAssemblies()
                .SelectMany(t => t.GetTypes())
                .Where(t => t.IsClass && namespaces.Contains(t.Namespace)).ToArray();
        }
    }
}