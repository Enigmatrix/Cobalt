namespace Cobalt.Common.Data.Migrations
{
    public interface IDbMigration
    {
        long Version { get; }
        void Run();
    }

    public abstract class MigrationBase : IDbMigration
    {
        public abstract long Version { get; }
        public abstract void Run();
    }
}