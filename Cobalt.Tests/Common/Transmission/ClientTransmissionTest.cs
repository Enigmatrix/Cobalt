using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Transmission;
using Xunit;

namespace Cobalt.Tests.Common.Transmission
{
    public class ClientTransmissionTest
    {
        [Fact]
        public void Test()
        {
            var client = new TransmissionClient();
            client.MessageReceived += (o, e) =>
            {
                
            };
            while (true) { }
        }
    }
}
