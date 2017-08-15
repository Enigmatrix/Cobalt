using System.Collections.Generic;
using System.Data;

namespace Cobalt.Common.Data.Migration
{
    public interface IDbMigrator
    {
        IDbConnection Connection { get; set; }

        List<MigrationBase> GetMigrations();
        void Migrate();
    }
}