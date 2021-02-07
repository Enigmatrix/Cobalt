using System;
using System.IO;
using System.Windows;
using Cobalt.Common.Communication;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Statistics;
using Microsoft.Data.Sqlite;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace Cobalt.Tray
{
    /// <summary>
    ///     Interaction logic for App.xaml
    /// </summary>
    public partial class App
    {
        protected override void ConfigureServices(IConfiguration configuration, IServiceCollection services)
        {
            // TODO move this to a common location
            services.AddSingleton(_ =>
            {
                var appdata = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                var conn = new SqliteConnection($"Data Source={Path.Join(appdata, "Cobalt", "data.db")}");
                conn.Open();
                new SqliteCommand("PRAGMA journal_mode='wal'", conn).ExecuteNonQuery();
                return conn;
            });
            services.AddSingleton<IDatabase, Database>();
            services.AddSingleton<IClient, Client>();
            services.AddSingleton<IQueries, Queries>();
            services.AddSingleton<IEntityManager, EntityManager>();

            services.AddSingleton<MainWindow>();
            services.AddSingleton<TrayViewModel>();
        }

        protected override void HostStartup(StartupEventArgs e)
        {
            var mainWindow = Host.Services.GetRequiredService<MainWindow>();
            mainWindow.Show();
        }
    }
}