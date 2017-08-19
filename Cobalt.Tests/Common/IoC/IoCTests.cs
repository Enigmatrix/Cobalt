namespace Cobalt.Tests.Common.IoC
{
    public class IoCTests
    {
        //assembly.getentryassembly() will return null, causing this to always fail within xunit
        /*
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
        */
    }
}