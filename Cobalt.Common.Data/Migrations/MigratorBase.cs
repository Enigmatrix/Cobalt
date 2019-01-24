using System.Collections.Generic;
using System.Linq;

namespace Cobalt.Common.Data.Migrations
{
    public interface IDbMigrator
    {
        void Run();
    }

    public abstract class MigratorBase : IDbMigrator
    {
        protected abstract List<IDbMigration> Migrations { get; }

        public void Run()
        {
            var currentVersion = GetVersion();
            foreach (var migration in Migrations
                .Where(x => x.Version > currentVersion)
                .OrderBy(x => x.Version))
                migration.Run();
        }

        protected abstract long GetVersion();
    }
}