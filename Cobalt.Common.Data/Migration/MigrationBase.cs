using System.Data;

namespace Cobalt.Common.Data.Migration
{
    public abstract class MigrationBase
    {
        protected MigrationBase(IDbConnection connection)
        {
        }

        public virtual int Order { get; } = 0;
        public abstract void ExecuteMigration();
    }
}