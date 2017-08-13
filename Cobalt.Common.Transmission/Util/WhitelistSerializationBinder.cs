using System;
using System.Collections.Generic;
using System.Linq;
using Cobalt.Common.Util;
using Newtonsoft.Json.Serialization;

namespace Cobalt.Common.Transmission.Util
{
    public class WhitelistSerializationBinder : ISerializationBinder
    {
        private readonly DefaultSerializationBinder _defaultBinder;
        private readonly HashSet<string> _whitelistedAssemblyNames;
        private readonly HashSet<string> _whitelistedTypeNames;

        public WhitelistSerializationBinder(params Type[] whitelist)
        {
            _defaultBinder = new DefaultSerializationBinder();
            _whitelistedTypeNames = new HashSet<string>(whitelist.Select(t => t.FullName));
            _whitelistedAssemblyNames = new HashSet<string>(whitelist.Select(t => t.Assembly.GetName().Name));
        }

        public Type BindToType(string assemblyName, string typeName)
        {
            if (!_whitelistedTypeNames.Contains(typeName) || !_whitelistedAssemblyNames.Contains(assemblyName))
                Throw.SecurityException($"Type `{typeName}` of assembly `{assemblyName}` not in whitelist!`");
            return _defaultBinder.BindToType(assemblyName, typeName);
        }

        public void BindToName(Type serializedType, out string assemblyName, out string typeName)
        {
            _defaultBinder.BindToName(serializedType, out assemblyName, out typeName);
        }
    }
}