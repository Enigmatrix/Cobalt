using Autofac;
using Caliburn.Micro.Autofac;
using Cobalt.Common.IoC;

namespace Cobalt.Common.UI
{
    public class Bootstrapper<T> : AutofacBootstrapper<T>
    {
        protected override void ConfigureContainer(ContainerBuilder builder)
        {
            IoCService.RegisterDependencies(builder);
            builder.RegisterBuildCallback(container =>
                IoCService.Instance = new IoCService(container));
        }
    }
}