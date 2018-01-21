using System;
using System.Windows;
using Autofac;
using Caliburn.Micro.Autofac;
using Cobalt.Common.IoC;

namespace Cobalt.Common.UI
{
    public abstract class Bootstrapper<T> : AutofacBootstrapper<T>
    {
        protected Bootstrapper()
        {
            Initialize();
        }

        protected override void ConfigureContainer(ContainerBuilder builder)
        {
            IoCService.RegisterDependencies(builder);
            builder.RegisterBuildCallback(container =>
                IoCService.Instance = new IoCService(container));
        }

        protected override void OnStartup(object sender, StartupEventArgs e)
        {
            DisplayRootViewFor<T>();
        }
    }
}