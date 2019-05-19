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
    public class IoCService : IDisposable
    {
        private static IoCService _instance;

        static IoCService()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.File($"./Logs/{Assembly.GetEntryAssembly().GetName().Name}-.log",
                    rollingInterval: RollingInterval.Day, shared: true)
                .CreateLogger();
            var folder = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            DbPath = Path.Combine(folder, "Cobalt.db");

        }

        public IoCService(Action<ContainerBuilder> register = null)
        {
            var builder = new ContainerBuilder();
            RegisterDependencies(builder);
            register?.Invoke(builder);
            Container = builder.Build();
        }

        public IContainer Container { get; }

        public static IoCService Instance
        {
            get { return _instance = _instance ?? new IoCService(); }
            set => _instance = value;
        }

        public void Dispose()
        {
            Container.Dispose();
        }

        public static Assembly[] AllAssemblies()
        {
            var a = Assembly.GetEntryAssembly().GetReferencedAssemblies()
                .Where(x => x.Name.Contains("Cobalt")).Select(Assembly.Load).ToList();
            a.Add(Assembly.GetEntryAssembly());
            return a.ToArray();
        }

        public static string DbPath { get; private set; }

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
                .Register(c => new SQLiteConnection($"Data Source={DbPath}").OpenAndReturn())
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