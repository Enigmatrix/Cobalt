﻿using System;
using System.Collections.Generic;
using System.Data;
using System.Linq;
using System.Reflection;

namespace Cobalt.Common.Data.Migration
{
    public abstract class MigratorBase : IDbMigrator
    {
        protected MigratorBase(IDbConnection connection)
        {
            Connection = connection;
        }

        public IDbConnection Connection { get; set; }

        public List<MigrationBase> GetMigrations()
        {
            return Assembly.GetExecutingAssembly().GetTypes()
                .Where(t => t.IsSubclassOf(typeof(MigrationBase)) && !t.IsAbstract &&
                            t.Namespace == GetType().Namespace)
                .Select(t => (MigrationBase) Activator.CreateInstance(t, Connection))
                .OrderBy(m => m.Order)
                .ToList();
        }

        public void Migrate()
        {
            var migrations = GetMigrations();
            var orders = migrations.Select(x => x.Order).ToList();
            var index = orders.BinarySearch(CurrentMigration());
            for (var i = index + 1; i < migrations.Count; i++)
                migrations[i].ExecuteMigration();
        }

        protected abstract int CurrentMigration();
    }
}