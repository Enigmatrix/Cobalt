using System;
using System.Collections.Generic;
using System.Data;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.IoC;
using Xunit;

namespace Cobalt.Tests.Common.IoC
{
    public class IoCTests
    {
        [Fact]
        public void TestScope()
        {
            var scope = IoCService.Instance.Resolve<IResourceScope>();
            var conn = scope.Resolve<IDbConnection>();
            //same within scope
            var conn1 = scope.Resolve<IDbConnection>();
            Assert.Equal(conn, conn1);

            //sam as global scope
            var nscope = IoCService.Instance.Resolve<IResourceScope>();
            Assert.Equal(conn, nscope.Resolve<IDbConnection>());

            var sscoep = scope.Subscope();
            var other = sscoep.Resolve<IDbConnection>();
            //not same as global scope
            Assert.NotEqual(conn, other);
            sscoep.Dispose();

            //not same as other scope
            var newsubscoep = scope.Subscope();
            Assert.NotEqual(other, newsubscoep.Resolve<IDbConnection>());
        }
    }
}
