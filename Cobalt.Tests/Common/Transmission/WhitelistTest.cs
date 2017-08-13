using System;
using System.IO;
using Cobalt.Common.Data;
using Cobalt.Common.Transmission.Util;
using Newtonsoft.Json;
using Xunit;

namespace Cobalt.Tests.Common.Transmission
{
    public class WhitelistTest
    {
        [Fact]
        public void DoesNotThrowForWhitelistedType()
        {
            var serialzier = Utilities.CreateSerializer();

            var json = @"{
		                    ""$type"": ""Cobalt.Common.Data.App, Cobalt.Common.Data"",
                            ""Name"": ""Awesome""
	                        }";

            var obj = serialzier.Deserialize(new JsonTextReader(new StringReader(json)));
            Assert.IsType<App>(obj);
            Assert.Equal("Awesome", ((App) obj).Name);
        }

        [Fact]
        public void ThrowsForNonWhitelistedType()
        {
            Assert.ThrowsAny<Exception>(() =>
            {
                var serialzier = Utilities.CreateSerializer();

                var json = @"{
		                    ""$type"": ""System.IO.FileInfo, System.IO.FileSystem"",
		                    ""fileName"": ""rce-test.txt"",
		                    ""IsReadOnly"": true
	                        }";

                serialzier.Deserialize(new JsonTextReader(new StringReader(json)));
            });
        }
    }
}