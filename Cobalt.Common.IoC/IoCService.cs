using System;
using System.Data;
using System.Data.SQLite;
using System.Linq;
using System.Reflection;
using Autofac;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.Transmission;
using Cobalt.Common.Util;

namespace Cobalt.Common.IoC
{
    public class IoCService
    {
        private static IoCService _instance;

        public IoCService()
        {
            var builder = new ContainerBuilder();
            RegisterDependencies(builder);
            Container = builder.Build();
        }

        public IoCService(IContainer container)
        {
            Container = container;
        }

        public IContainer Container { get; }

        public static IoCService Instance
        {
            get { return _instance = _instance ?? new IoCService(); }
            set => _instance = value;
        }

        public static Assembly[] AllAssemblies()
        {
            var a = Assembly.GetEntryAssembly().GetReferencedAssemblies()
                .Where(x => x.Name.Contains("Cobalt")).Select(Assembly.Load).ToList();
            a.Add(Assembly.GetEntryAssembly());
            return a.ToArray();
        }

        public static void RegisterDependencies(ContainerBuilder builder)
        {
            builder.RegisterAssemblyTypes(AllAssemblies())
                .Where(type => !string.IsNullOrWhiteSpace(type.Namespace) && type.Namespace.StartsWith("Cobalt"))
                .PreserveExistingDefaults()
                .AsSelf()
                //but we also make interfaces to be implemented
                .AsImplementedInterfaces()
                .InstancePerDependency();

            builder
                .Register(c => new SQLiteConnection("Data Source=dat.db").OpenAndReturn())
                .As<IDbConnection>()
                .InstancePerLifetimeScope();

            builder
                .RegisterType<SqliteRepository>()
                .As<IDbRepository>()
                .InstancePerLifetimeScope();

            //register single instance transmission client
            builder.RegisterType<TransmissionClient>()
                .As<ITransmissionClient>()
                .SingleInstance();

            builder
                .Register(c => new GlobalClock(TimeSpan.FromMilliseconds(35)))
                .As<IGlobalClock>()
                .InstancePerLifetimeScope();
        }

        public T Resolve<T>()
        {
            return Container.Resolve<T>();
        }
    }
}