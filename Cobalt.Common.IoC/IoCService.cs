using System;
using System.Data.SQLite;
using System.IO;
using System.Linq;
using System.Reflection;
using Autofac;
using Cobalt.Common.Data.Migrations;
using Cobalt.Common.Data.Repositories;
using Cobalt.Common.Transmission;
using Serilog;

namespace Cobalt.Common.IoC
{
    public class IoCService
    {
        private static IoCService _instance;

        static IoCService()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.File($"./Logs/{Assembly.GetEntryAssembly().GetName().Name}-.log",
                    rollingInterval: RollingInterval.Day, shared: true)
                .CreateLogger();
        }

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

            var folder = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            var path = Path.Combine(folder, "Cobalt.db");

            builder
                .Register(c => new SQLiteConnection($"Data Source={path}").OpenAndReturn())
                .As<SQLiteConnection>()
                .SingleInstance();

            builder
                .RegisterType<SqliteMigrator>()
                .As<IDbMigrator>()
                .SingleInstance();

            builder
                .RegisterType<SqliteRepository>()
                .As<IDbRepository>()
                .SingleInstance();

            builder.RegisterType<TransmissionClient>()
                .As<ITransmissionClient>()
                .SingleInstance();

            /*builder
                .Register(c => new GlobalClock(TimeSpan.FromMilliseconds(237)))
                .As<IGlobalClock>()
                .InstancePerLifetimeScope();*/
        }

        public T Resolve<T>()
        {
            return Container.Resolve<T>();
        }

        public object Resolve(Type t)
        {
            return Container.Resolve(t);
        }
    }
}