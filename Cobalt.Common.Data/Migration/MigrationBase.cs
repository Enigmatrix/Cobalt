using System;
using System.Collections.Generic;
using System.Data;
using System.Data.Common;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Data.Migration
{
    public abstract class MigrationBase
    {
        public abstract void ExecuteMigration();
        public virtual int Order { get; } = 0;
        protected MigrationBase(IDbConnection connection){}
    }
}
