using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Cobalt.Common.Data.Migrations
{
    public interface IDbMigrator
    {
        void Run();
    }
    public abstract class MigratorBase : IDbMigrator
    {
        protected abstract List<IDbMigration> Migrations { get; }

        protected abstract long GetVersion();

        public void Run()
        {
            var currentVersion = GetVersion();
            foreach (var migration in Migrations
                .Where(x => x.Version > currentVersion)
                .OrderBy(x => x.Version))
            {
                migration.Run();
            }
        }
    }
}
